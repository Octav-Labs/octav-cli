```
 ██████╗  ██████╗████████╗ █████╗ ██╗   ██╗
██╔═══██╗██╔════╝╚══██╔══╝██╔══██╗██║   ██║
██║   ██║██║        ██║   ███████║██║   ██║
██║   ██║██║        ██║   ██╔══██║╚██╗ ██╔╝
╚██████╔╝╚██████╗   ██║   ██║  ██║ ╚████╔╝
 ╚═════╝  ╚═════╝   ╚═╝   ╚═╝  ╚═╝  ╚═══╝
```

# Octav CLI

Command-line interface for the [Octav](https://octav.fi) cryptocurrency portfolio API. Query portfolio data, transactions, net worth, and historical snapshots across 20+ blockchains — all from your terminal.

## Installation

### Shell script

```bash
curl -sSf https://raw.githubusercontent.com/Octav-Labs/octav-cli/main/install.sh | sh
```

### Cargo

```bash
cargo install octav
```

### From source

```bash
git clone https://github.com/Octav-Labs/octav-cli.git
cd octav-cli
cargo build --release
cp target/release/octav /usr/local/bin/
```

## Quick Start

```bash
# Store your API key (get one at https://octav.fi/api)
octav auth set-key YOUR_API_KEY

# Launch the interactive dashboard
octav dashboard --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68

# Or query your portfolio as JSON
octav portfolio get --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68

# Check credit balance
octav credits
```

## Dashboard

Interactive terminal UI for exploring your portfolio. Shows holdings with token icons, protocol breakdown, chain distribution, and transactions.

```bash
octav dashboard --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

Multiple addresses:

```bash
octav dashboard --addresses 0xABC...123,0xDEF...456
```

| Key | Action |
|-----|--------|
| `Tab` / `1-4` | Switch screens (Overview, Holdings, Protocols, Transactions) |
| `j` / `k` | Scroll down / up |
| `g` / `G` | Jump to top / bottom |
| `Enter` | Drill into protocol details |
| `Esc` | Go back |
| `r` | Refresh data |
| `q` | Quit |

Token images are cached locally in `~/.octav/cache/images/`.

## Commands

### Authentication

#### `octav auth set-key <KEY>`

Store API key in `~/.octav/config.json`.

```bash
octav auth set-key sk_live_abc123
```

#### `octav auth show`

Show current API key source and masked value.

```bash
octav auth show
```

### Portfolio

#### `octav portfolio get` — 1 credit/address

Get full portfolio including DeFi positions.

```bash
octav portfolio get --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

#### `octav portfolio wallet` — 1 credit/address

Get wallet holdings only (excludes DeFi protocols).

```bash
octav portfolio wallet --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

#### `octav portfolio nav` — 1 credit/address

Get net asset value in a specified currency.

```bash
octav portfolio nav --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --currency EUR
```

Supported currencies: `USD` (default), `EUR`, `GBP`, `JPY`, `CNY`.

#### `octav portfolio token-overview` — 1 credit/address

Get aggregated token distribution across all chains for a specific date.

```bash
octav portfolio token-overview --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --date 2024-06-01
```

### Transactions

#### `octav transactions get` — 1 credit/address

Query transaction history with filtering and pagination.

```bash
# Basic query
octav transactions get --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68

# With filters
octav transactions get \
  --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 \
  --chain ethereum \
  --type swap \
  --start-date 2024-01-01 \
  --end-date 2024-06-30 \
  --offset 0 \
  --limit 100
```

| Flag | Description | Default |
|------|-------------|---------|
| `--chain` | Filter by chain | all |
| `--type` | Filter by transaction type | all |
| `--start-date` | Start date (YYYY-MM-DD) | — |
| `--end-date` | End date (YYYY-MM-DD) | — |
| `--offset` | Pagination offset | `0` |
| `--limit` | Results per page (max 250) | `50` |

#### `octav transactions sync` — 1 credit/address

Trigger transaction synchronization.

```bash
octav transactions sync --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

### Historical

#### `octav historical get` — 1 credit/address

Get portfolio snapshot for a specific date.

```bash
octav historical get --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --date 2024-01-01
```

#### `octav historical subscribe-snapshot` — 1 credit/address

Subscribe to automatic daily portfolio snapshots.

```bash
octav historical subscribe-snapshot \
  --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 \
  --description "Main wallet daily snapshot"
```

### Metadata

#### `octav status` — FREE

Check sync status for addresses.

```bash
octav status --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

#### `octav credits` — FREE

Check API credit balance.

```bash
octav credits
```

### Specialized

#### `octav airdrop` — 1 credit

Check Solana airdrop eligibility.

```bash
octav airdrop --address 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

#### `octav polymarket` — 1 credit

Get Polymarket prediction market positions.

```bash
octav polymarket --address 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

### Agent (x402 payment)

#### `octav agent wallet`

Get wallet holdings via x402 payment protocol (for AI agents).

```bash
octav agent wallet --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

#### `octav agent portfolio`

Get full portfolio via x402 payment protocol (for AI agents).

```bash
octav agent portfolio --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
```

## Multiple Addresses

Most commands accept multiple addresses as a comma-separated list:

```bash
octav portfolio get --addresses 0xABC...123,0xDEF...456,7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

Maximum 10 addresses per request.

## Authentication

API key is resolved in this order (first wins):

1. `--api-key` flag
2. `OCTAV_API_KEY` environment variable
3. `~/.octav/config.json` config file

```bash
# Flag (highest precedence)
octav credits --api-key sk_live_abc123

# Environment variable
export OCTAV_API_KEY=sk_live_abc123
octav credits

# Config file (set once, used automatically)
octav auth set-key sk_live_abc123
```

## Output Format

All output is JSON. Pretty-printed by default, compact with `--raw`.

```bash
# Pretty-printed (default)
octav credits
# => {
# =>   "credits": 42
# => }

# Compact JSON
octav credits --raw
# => {"credits":42}
```

The `--raw` flag also disables portfolio field stripping (returns full API response).

### Error format

Errors are returned as JSON on stdout with a non-zero exit code:

```json
{
  "error": {
    "type": "auth",
    "message": "Invalid API key",
    "status": 401
  }
}
```

## Supported Address Formats

- **EVM**: `0x` followed by 40 hex characters (Ethereum, Polygon, Arbitrum, Base, etc.)
- **Solana**: 32-44 character base58 strings

## Credit Costs

| Endpoint | Cost |
|----------|------|
| `portfolio get` | 1 credit/address |
| `portfolio wallet` | 1 credit/address |
| `portfolio nav` | 1 credit/address |
| `portfolio token-overview` | 1 credit/address |
| `transactions get` | 1 credit/address |
| `transactions sync` | 1 credit/address |
| `historical get` | 1 credit/address |
| `historical subscribe-snapshot` | 1 credit/address |
| `status` | FREE |
| `credits` | FREE |
| `airdrop` | 1 credit |
| `polymarket` | 1 credit |
| `agent wallet` | x402 payment |
| `agent portfolio` | x402 payment |

Purchase credits at [octav.fi](https://octav.fi).

## Supported Chains

Ethereum, Solana, Arbitrum, Base, Polygon, Optimism, BNB Chain, Avalanche, Fantom, Cronos, Gnosis, Celo, Moonbeam, Moonriver, Harmony, Aurora, Metis, Boba, Fuse, Evmos, Kava, and more.

## License

MIT

## Links

- [Octav Website](https://octav.fi)
- [Octav API Docs](https://docs.octav.fi)
- [Octav MCP Server](https://github.com/Octav-Labs/octav-api-mcp)
