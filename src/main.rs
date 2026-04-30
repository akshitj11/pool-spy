use serde::Deserialize;

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
    timestamp: i64,
    #[serde(rename = "token0Price")]
    token0_price: f64,
    #[serde(rename = "token1Price")]
    token1_price: f64,
}


#[derive(Deserialize, Debug)]
struct Pool {
    id: String,
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

impl Pool {
    // Get the latest price from price history
    fn current_price(&self) -> f64 {
        self.price_history
            .last()
            .map(|p| p.token0_price)
            .unwrap_or(0.0)
    }

    // Calculate 24h price change %
    fn price_change_24h(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }
        let first = self.price_history.first().unwrap().token0_price;
        let last = self.price_history.last().unwrap().token0_price;
        ((last - first) / first) * 100.0
    }

    // Fee tier as human readable string
    fn fee_percent(&self) -> String {
        format!("{:.2}%", self.fee_tier / 10000.0)
    }

    fn display(&self) {
        let price = self.current_price();
        let change = self.price_change_24h();
        let arrow = if change >= 0.0 { "↑" } else { "↓" };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Pair:      {}/{}", self.token0.symbol, self.token1.symbol);
        println!("  Price:     ${:.2}  {} {:.2}% (24h)",
            price, arrow, change.abs());
        println!("  TVL:       ${:.0}", self.total_liquidity.value);
        println!("  Fee Tier:  {}", self.fee_percent());
        println!("  Txns:      {}", self.tx_count);
        println!("  Reserves:  {} {} / {:.2} {}",
            format_number(self.token0_supply), self.token0.symbol,
            self.token1_supply, self.token1.symbol);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

// Format large numbers nicely: 71456294 → 71.5M
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

#[tokio::main]
async fn main() {
    println!("\npool-spy — Live Uniswap V3 Pools\n");

    let query = r#"
    {
        topV3Pools(first: 5, chain: ETHEREUM) {
            id
            token0 { symbol }
            token1 { symbol }
            totalLiquidity { value }
            feeTier
            txCount
            token0Supply
            token1Supply
            priceHistory(duration: DAY) {
                timestamp
                token0Price
                token1Price
            }
        }
    }
    "#;

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

    for pool in &result.data.top_v3_pools {
        pool.display();
    }

    println!("\nData fetched live from Uniswap V3\n");
}