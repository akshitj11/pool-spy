struct Pool {
    token0: String,
    token1: String,
    tvl_usd: f64,
    fees_7d_usd: f64,
    fee_tier: String,
}

impl Pool {
    fn apy(&self) -> f64 {
        (self.fees_7d_usd / self.tvl_usd) * 52.0 * 100.0
    }

    fn display(&self) {
        println!("  Pair:{}/{}", self.token0, self.token1);
        println!("  Fee Tier:{}", self.fee_tier);
        println!("  TVL:${:.0}", self.tvl_usd);
        println!("  APY:{:.2}%", self.apy());
    }
}

fn main() {
    let pools = vec![
        Pool {
            token0: String::from("ETH"),
            token1: String::from("USDC"),
            tvl_usd: 1_000_000.0,
            fees_7d_usd: 10_000.0,
            fee_tier: String::from("0.3%"),
        },
        Pool {
            token0: String::from("BTC"),
            token1: String::from("USDC"),
            tvl_usd: 500_000.0,
            fees_7d_usd: 8_000.0,
            fee_tier: String::from("0.05%"),
        },
        Pool {
            token0: String::from("ARB"),
            token1: String::from("ETH"),
            tvl_usd: 250_000.0,
            fees_7d_usd: 12_000.0,
            fee_tier: String::from("0.3%"),
        },
    ];

    println!("\n pool spy\n");

    for pool in &pools {
        pool.display();
    }
}