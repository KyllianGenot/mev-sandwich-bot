# MEV Sandwich Bot

A high-performance MEV (Maximum Extractable Value) sandwich bot built in Rust, targeting Uniswap V2/V3 swaps on Ethereum Sepolia testnet.

## Overview

This bot monitors the mempool for pending DEX swap transactions and executes sandwich attacks:
1. **Frontrun**: Buy the token before the victim's transaction
2. **Victim tx executes**: Price moves due to victim's swap
3. **Backrun**: Sell the token at a higher price for profit

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     MEV Sandwich Bot                        │
├─────────────────────────────────────────────────────────────┤
│  Mempool Listener (WebSocket)                               │
│       │                                                     │
│       ▼                                                     │
│  Transaction Decoder (V2/V3 swap detection)                 │
│       │                                                     │
│       ▼                                                     │
│  Simulator (Uniswap V3 Quoter profitability check)          │
│       │                                                     │
│       ▼                                                     │
│  Bundle Executor ──────► SandwichExecutor.sol (on-chain)    │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

- **Rust** (1.75+): [Install Rust](https://rustup.rs/)
- **Foundry**: [Install Foundry](https://book.getfoundry.sh/getting-started/installation)
- **RPC Provider**: WebSocket-enabled Ethereum node (Alchemy, Infura, QuickNode, etc.)
- **Sepolia ETH**: For gas fees
- **Sepolia WETH**: For trading capital

## Installation

### 1. Clone and Install Dependencies

```bash
git clone git@github.com:KyllianGenot/mev-sandwich-bot.git
cd mev-sandwich-bot

# Install Foundry dependencies
forge install

# Build Rust project
cargo build --release
```

### 2. Environment Setup

Create a `.env` file in the project root:

```env
# Ethereum RPC endpoints (Sepolia)
RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
RPC_WS_URL=wss://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# Your wallet private key (without 0x prefix)
PRIVATE_KEY=your_private_key_here

# Chain ID (11155111 for Sepolia)
CHAIN_ID=11155111

# Deployed SandwichExecutor contract address (set after deployment)
EXECUTOR_ADDRESS=0x...

# Optional: For contract verification
ETHERSCAN_API_KEY=your_etherscan_api_key
```

> ⚠️ **Security Warning**: Never commit your `.env` file or share your private key. Add `.env` to `.gitignore`.

### 3. Deploy the Smart Contract

Deploy the `SandwichExecutor` contract to Sepolia:

```bash
# Load environment variables
source .env

# Deploy contract
forge script script/Deploy.s.sol:DeployScript \
    --rpc-url $RPC_URL \
    --broadcast \
    --verify
```

Save the deployed contract address and update `EXECUTOR_ADDRESS` in your `.env` file.

### 4. Fund the Executor Contract

The executor contract needs WETH to perform frontrun swaps:

```bash
# Wrap ETH to WETH on Sepolia
# WETH address: 0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14

# Transfer WETH to your deployed executor contract
# Use your preferred method (Etherscan, cast, etc.)
```

Using `cast` (Foundry):

```bash
# Approve and transfer WETH to executor
cast send 0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14 \
    "transfer(address,uint256)" \
    $EXECUTOR_ADDRESS \
    1000000000000000000 \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY
```

## Running the Bot

### Development Mode

```bash
cargo run
```

### Production Mode

```bash
cargo run --release
```

### With Debug Logging

```bash
RUST_LOG=mev_sandwich_bot=debug cargo run --release
```

## Configuration

### Supported DEX Routers (Sepolia)

| Router | Address |
|--------|---------|
| Uniswap V3 SwapRouter02 | `0x3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48E` |
| Uniswap V2 Router | `0xeE567Fe1712Faf6149d80dA1E6934E354124CfE3` |
| Universal Router | `0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD` |
| Universal Router 2 | `0x5E325eDA8064b456f4781070C0738d849c824258` |

### Key Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| Min victim amount | 0.001 ETH | Minimum swap size to target |
| Frontrun percentage | 30% | Portion of victim's swap to frontrun |
| Fee tiers checked | 500, 3000, 10000 | Uniswap V3 fee tiers (bps) |

## Contract Functions

### SandwichExecutor.sol

| Function | Description |
|----------|-------------|
| `frontrun()` | Execute buy swap before victim |
| `backrun()` | Execute sell swap after victim |
| `getBalance(token)` | Check token balance in contract |
| `withdraw(token, amount)` | Withdraw specific amount |
| `withdrawAll(token)` | Withdraw entire token balance |
| `withdrawETH()` | Withdraw native ETH |

## Project Structure

```
mev-sandwich-bot/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Environment configuration
│   ├── mempool/
│   │   ├── listener.rs      # WebSocket mempool listener
│   │   └── decoder.rs       # Transaction decoder
│   ├── analysis/
│   │   └── simulator.rs     # Profitability simulation
│   ├── execution/
│   │   └── bundle.rs        # Trade execution logic
│   └── contracts/
│       └── mod.rs           # Contract bindings
├── contracts/
│   └── SandwichExecutor.sol # On-chain executor
├── script/
│   └── Deploy.s.sol         # Deployment script
├── Cargo.toml               # Rust dependencies
└── foundry.toml             # Foundry configuration
```

## Troubleshooting

### "Executor has 0 WETH"
Fund your executor contract with WETH before running the bot.

### WebSocket Connection Failed
- Verify your `RPC_WS_URL` is correct and uses `wss://`
- Ensure your RPC provider supports WebSocket connections
- Check if you've exceeded rate limits

### Transactions Reverting
- Ensure the executor contract has sufficient token balance
- Check that you're the owner of the executor contract
- Verify gas prices are competitive

### No Opportunities Found
- Sepolia testnet has lower activity than mainnet
- Adjust `min_victim_amount` threshold if needed
- Verify the mempool subscription is working

## Disclaimer

⚠️ **Educational Purpose Only**

This bot is designed for learning and experimentation on testnets. MEV extraction on mainnet involves significant risks:

- Financial losses from failed transactions
- Competition from professional MEV searchers
- Potential for front-running by other actors
- Gas costs can exceed profits

**Do not use on mainnet without thorough understanding of the risks involved.**

## License

MIT
