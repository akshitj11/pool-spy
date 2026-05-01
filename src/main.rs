use serde::Deserialize;
use clap::{Parser, Subcommand};

// ─── CLI STRUCTURE ───────────────────────────────────────────

#[derive(Parser)]
#[command(name = "pool-spy")]
#[command(about = "Live Uniswap V3 pool data in your terminal")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show top pools by TVL
    Top {
        /// Number of pools to show (default: 5)
        #[arg(short, long, default_value_t = 5)]
        limit: u32,
    },
    /// Show details for a specific pool pair
    Info {
        /// Token pair e.g. USDC/WETH
        pair: String,
    },
}

// ─── API DATA STRUCTURES ─────────────────────────────────────

#[derive(Deserialize, Debug)]
struct Token {
    symbol: String,
}

#[derive(Deserialize, Debug)]
struct Liquidity {
    value: f64,
}

#[derive(Deserialize, Debug)]
struct PricePoint {
    #[serde(rename = "token0Price")]
    token0_price: f64,
}

#[derive(Deserialize, Debug)]
struct Pool {
    token0: Token,
    token1: Token,
    #[serde(rename = "totalLiquidity")]
    total_liquidity: Liquidity,
    #[serde(rename = "feeTier")]
    fee_tier: f64,
    #[serde(rename = "txCount")]
    tx_count: i64,
    #[serde(rename = "token0Supply")]
    token0_supply: f64,
    #[serde(rename = "token1Supply")]
    token1_supply: f64,
    #[serde(rename = "priceHistory")]
    price_history: Vec<PricePoint>,
}

#[derive(Deserialize, Debug)]
struct TopPools {
    #[serde(rename = "topV3Pools")]
    top_v3_pools: Vec<Pool>,
}

#[derive(Deserialize, Debug)]
struct GraphQLData {
    data: TopPools,
}

// ─── POOL METHODS ─────────────────────────────────────────────

impl Pool {
    fn pair_name(&self) -> String {
        format!("{}/{}", self.token0.symbol, self.token1.symbol)
    }

    fn current_price(&self) -> f64 {
        self.price_history
            .last()
            .map(|p| p.token0_price)
            .unwrap_or(0.0)
    }

    fn price_change_24h(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }
        let first = self.price_history.first().unwrap().token0_price;
        let last = self.price_history.last().unwrap().token0_price;
        ((last - first) / first) * 100.0
    }

    fn fee_percent(&self) -> String {
        format!("{:.2}%", self.fee_tier / 10000.0)
    }

    fn display(&self) {
        let price = self.current_price();
        let change = self.price_change_24h();
        let arrow = if change >= 0.0 { "↑" } else { "↓" };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Pair:      {}", self.pair_name());
        println!("  Price:     ${:.2}  {} {:.2}% (24h)", price, arrow, change.abs());
        println!("  TVL:       ${:.0}", self.total_liquidity.value);
        println!("  Fee Tier:  {}", self.fee_percent());
        println!("  Txns:      {}", self.tx_count);
        println!("  Reserves:  {} {} / {:.2} {}",
            format_number(self.token0_supply), self.token0.symbol,
            self.token1_supply, self.token1.symbol);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

// ─── HELPER FUNCTIONS ─────────────────────────────────────────

fn format_number(n: f64) -> String {
    if n >= 1_000_000_000.0 {
        format!("{:.1}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else {
        format!("{:.2}", n)
    }
}

async fn fetch_pools(limit: u32) -> Vec<Pool> {
    let query = format!(r#"
    {{
        topV3Pools(first: {}, chain: ETHEREUM) {{
            token0 {{ symbol }}
            token1 {{ symbol }}
            totalLiquidity {{ value }}
            feeTier
            txCount
            token0Supply
            token1Supply
            priceHistory(duration: DAY) {{
                token0Price
            }}
        }}
    }}
    "#, limit);

    let client = reqwest::Client::new();

    let response = client
        .post("https://interface.gateway.uniswap.org/v1/graphql")
        .header("Content-Type", "application/json")
        .header("Origin", "https://app.uniswap.org")
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .expect("Failed to send request");

    let result: GraphQLData = response
        .json()
        .await
        .expect("Failed to parse response");

    result.data.top_v3_pools
}

// ─── MAIN ─────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Top { limit } => {
            println!("\n pool-spy — Top {} Uniswap V3 Pools\n", limit);
            let pools = fetch_pools(limit).await;
            for pool in &pools {
                pool.display();
            }
            println!("\n {} pools fetched live from Uniswap V3\n", pools.len());
        }

        Commands::Info { pair } => {
            println!("\n pool-spy — Looking up {}\n", pair);
            let pools = fetch_pools(20).await;

            let parts: Vec<&str> = pair.split('/').collect();
            if parts.len() != 2 {
                println!("Invalid pair format. Use: USDC/WETH");
                return;
            }

            let token0 = parts[0].to_uppercase();
            let token1 = parts[1].to_uppercase();

            let found: Vec<&Pool> = pools
                .iter()
                .filter(|p| {
                    (p.token0.symbol == token0 && p.token1.symbol == token1)
                    || (p.token0.symbol == token1 && p.token1.symbol == token0)
                })
                .collect();

            if found.is_empty() {
                println!("No pool found for {}. Try: USDC/WETH, WBTC/WETH", pair);
            } else {
                for pool in found {
                    pool.display();
                }
            }
        }
    }
}