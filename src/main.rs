use serde::Deserialize;
use clap::{Parser, Subcommand};
use tokio::time::{sleep, Duration};
use std::io::{self,Write};

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
    /// refreshing the pool evey N seconds , so it is updated regurlarly
    Watch{
        pair: String,
        #[arg(short,long,default_value_t=30)]
        interval: u64,
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

            // Print numbered list
            for (i, pool) in pools.iter().enumerate() {
                let price = pool.current_price();
                let change = pool.price_change_24h();
                let arrow = if change >= 0.0 { "↑" } else { "↓" };
                println!(
                    "  [{}] {:<12} TVL: ${:<12} Price: ${:.2} {} {:.2}%",
                    i + 1,
                    pool.pair_name(),
                    format_number(pool.total_liquidity.value),
                    price,
                    arrow,
                    change.abs()
                );
            }

            // Ask user to pick a pool
            loop {
                print!("\nEnter pool number for details (or q to quit): ");
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                if input == "q" || input == "Q" {
                    println!("Bye!");
                    break;
                }

                match input.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= pools.len() => {
                        println!();
                        pools[n - 1].display();
                    }
                    _ => {
                        println!(" Invalid input. Enter a number between 1 and {}", pools.len());
                    }
                }
            }
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

        Commands::Watch {pair,interval} => {
            let parts: Vec<&str>=pair.split('/').collect();
            if parts.len() !=2{
                println!("invalid pair formate. use:USDC/WETH");
            }

            let token0=parts[0].to_uppercase();
            let token1=parts[1].to_uppercase();

            println!("\n👁  Watching {}/{} — refreshing every {}s", token0, token1, interval);
            println!("Press Ctrl+C to stop\n");
            

            let mut last_price: Option<f64> = None;
            let mut refresh_count = 0;

            loop {
                refresh_count += 1;
                let pools = fetch_pools(20).await;

                let found: Option<&Pool> = pools
                    .iter()
                    .find(|p| {
                        (p.token0.symbol == token0 && p.token1.symbol == token1)
                        || (p.token0.symbol == token1 && p.token1.symbol == token0)
                    });

                match found {
                    None => {
                        println!("Pool not found for {}/{}", token0, token1);
                        break;
                    }
                    Some(pool) => {
                        let price = pool.current_price();

                        // Calculate change since last refresh
                        let change_since_last = match last_price {
                            Some(prev) => {
                                let change = ((price - prev) / prev) * 100.0;
                                let arrow = if change >= 0.0 { "↑" } else { "↓" };
                                format!("{} {:.3}% since last refresh", arrow, change.abs())
                            }
                            None => String::from("first reading"),
                        };

                        // Clear screen and reprint
                        print!("\x1B[2J\x1B[1;1H");

                        println!("👁  pool-spy watch — {}/{}", token0, token1);
                        println!("Refresh #{} | every {}s | Ctrl+C to stop\n", refresh_count, interval);

                        pool.display();

                        println!("\n   {}", change_since_last);
                        println!("   Next refresh in {}s...", interval);

                        last_price = Some(price);
                    }
                }

                sleep(Duration::from_secs(interval)).await;
            }
        }
    }
}
        