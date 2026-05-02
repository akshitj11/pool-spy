# pool-spy

CLI tool to monitor Uniswap V3 pools in real time. Built with Rust.

## Install

```bash
git clone https://github.com/akshitj11/pool-spy
cd pool-spy
cargo build --release
```

## Commands

### top
Show top pools by TVL.

```bash
cargo run -- top
cargo run -- top --limit 10        # default: 5
cargo run -- top --sort txns       # default: tvl
cargo run -- top --output json     # default: table
```

### info
Look up a specific pool by pair.

```bash
cargo run -- info USDC/WETH
cargo run -- info WBTC/WETH
```

### watch
Watch a pool live with auto-refresh.

```bash
cargo run -- watch USDC/WETH
cargo run -- watch USDC/WETH --interval 10     # default: 30 seconds
cargo run -- watch USDC/WETH --alert-price 5   # alert on 5% price move
cargo run -- watch USDC/WETH --interval 10 --alert-price 2
```

## Stack

- Rust
- clap — CLI parsing
- reqwest + tokio — async HTTP
- serde — JSON parsing
- anyhow — error handling
