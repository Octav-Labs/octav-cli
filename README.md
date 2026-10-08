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

## Updating

```bash
octav --version          # show the installed version
octav update --check     # check for a newer release
octav update             # install the latest release
```

`octav update` replaces the binary installed by the shell script. If you installed with Cargo, run `cargo install octav` instead; for source builds, `git pull` and rebuild. `octav update` tells you which applies.

Before installing, `octav update` checks the download against the SHA-256 checksum published with the release and stops without changing anything if it doesn't match.

When run in an interactive terminal, octav checks GitHub for a new release at most once a day and prints a notice to stderr if one is available. The check never runs in scripts, CI or other non-interactive use, and never changes command output. Set `OCTAV_NO_UPDATE_CHECK=1` to turn it off.

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

| Flag | Description |
|------|-------------|
| `--explorer-urls` | Include blockchain explorer URLs for assets and transactions |
| `--no-wait-for-sync` | Return cached data immediately instead of waiting for a fresh sync |

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

#### `octav portfolio at-block` — 1 credit (requires add-on)

Get a single EVM address's portfolio valued at a specific block on `ethereum`, `linea` or `monad`. Requires the Portfolio at Block add-on on your API key.

```bash
octav portfolio at-block --address 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --chain ethereum --block 20000000
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
  --chain ethereum,arbitrum \
  --type SWAP \
  --hide-spam \
  --start-date 2024-01-01 \
  --end-date 2024-06-30 \
  --offset 0 \
  --limit 100
```

| Flag | Description | Default |
|------|-------------|---------|
| `--chain` | Filter by chain keys, comma-separated (see `octav chains list`) | all |
| `--type` | Filter by transaction types, comma-separated (e.g. `SWAP,DEPOSIT`) | all |
| `--protocol` | Filter by protocol keys, comma-separated | all |
| `--interacting-address` | Filter by counterparty addresses, comma-separated | all |
| `--search` | Full-text search over token symbols, names and addresses | — |
| `--token-id` | Filter by NFT token ID | — |
| `--start-date` | Start date (YYYY-MM-DD) | — |
| `--end-date` | End date (YYYY-MM-DD) | — |
| `--sort` | `asc` or `desc` by timestamp | `desc` |
| `--hide-spam` | Exclude spam transactions | off |
| `--hide-dust` | Exclude dust transactions | off |
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

#### `octav chains list` — FREE

List supported chains.

```bash
octav chains list
```

#### `octav chains protocols` — FREE

List protocols on a chain, paginated.

```bash
octav chains protocols --chain ethereum --page 1 --limit 20
```

#### `octav contract-protocol` — 5 credits (refunded if not found)

Resolve a contract address to its DeFi protocol. Omit `--chain` to search all chains.

```bash
octav contract-protocol --contract 0x1111111254eeb25477b68fb85ed929f73a960582 --chain arbitrum
```

### Write Commands

Address book and bundle commands are the only ones that change data on your Octav account. Every other command is read-only.

| Command | Effect | Needs `--yes` |
|---------|--------|---------------|
| `addressbook add` | Adds addresses | No |
| `addressbook rename` | Overwrites a label | Yes |
| `addressbook remove` | Deletes an address | Yes |
| `bundles create` | Creates a bundle | No |
| `bundles add-address` | Adds an address to a bundle | No |
| `bundles rename` | Overwrites a bundle name | Yes |
| `bundles remove-address` | Removes an address from a bundle | Yes |
| `bundles delete` | Deletes a bundle | Yes |

Commands that delete or overwrite data refuse to run without `--yes` and exit with an error of type `confirmation_required`, without sending any request:

```json
{
  "error": {
    "type": "confirmation_required",
    "message": "This permanently deletes bundle abc123 from your Octav account (its addresses stay in the address book). Re-run with --yes to confirm."
  }
}
```

AI agents should treat `confirmation_required` as a stop: check with the user before re-running with `--yes`.

### Address Book — 1 credit per call

Manage the addresses saved to your account. Addresses already in the book keep their existing label when re-added.

```bash
octav addressbook list
octav addressbook add --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --label "Main wallet"
octav addressbook add --addresses 0xABC...123,0xDEF...456     # up to 100, no labels
octav addressbook rename --address 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --label "Treasury" --yes
octav addressbook remove --address 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --yes
```

### Bundles — 1 credit per call

Bundles are named groups of address book entries. Addresses must already be in the address book.

```bash
octav bundles list
octav bundles get --id <BUNDLE_ID>
octav bundles create --name "Treasury" --addresses 0xABC...123,0xDEF...456
octav bundles rename --id <BUNDLE_ID> --name "Core Treasury" --yes
octav bundles add-address --id <BUNDLE_ID> --address 0xABC...123
octav bundles remove-address --id <BUNDLE_ID> --address 0xABC...123 --yes
octav bundles delete --id <BUNDLE_ID> --yes      # addresses stay in the address book
```

### Virtual Users (requires Pro)

#### `octav virtual-users list` — 1 credit

```bash
octav virtual-users list
```

#### `octav virtual-users portfolio` — 1 credit/address

Get portfolios for virtual users (addresses from `virtual-users list`, in the form `virtual:<id>`). Accepts the same `--explorer-urls` and `--no-wait-for-sync` flags as `portfolio get`.

```bash
octav virtual-users portfolio --addresses virtual:abc123,virtual:def456 --aggregated
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

#### `octav approvals` — 1 credit

Get token approvals for an EVM wallet on one chain, paginated with `--limit` (1-100) and `--cursor`.

```bash
octav approvals --address 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --chain ethereum
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

#### `octav agent nav` / `agent status` / `agent chains`

Net asset value, sync status and supported chains via x402 payment.

```bash
octav agent nav --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --currency EUR
octav agent status --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
octav agent chains
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
octav status --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68
# => [
# =>   {
# =>     "address": "0x742d35cc6634c0532925a3b844bc9e7595f2bd68",
# =>     "syncInProgress": false,
# =>     ...
# =>   }
# => ]

# Compact JSON
octav status --addresses 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD68 --raw
# => [{"address":"0x742d35cc6634c0532925a3b844bc9e7595f2bd68","syncInProgress":false,...}]
```

`octav credits` prints a bare number (e.g. `4872`), so use it directly rather than with `jq '.credits'`.

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
| `portfolio at-block` | 1 credit (requires add-on) |
| `status` | FREE |
| `credits` | FREE |
| `chains list` | FREE |
| `chains protocols` | FREE |
| `contract-protocol` | 5 credits (refunded if not found) |
| `addressbook *` | 1 credit |
| `bundles *` | 1 credit |
| `virtual-users list` | 1 credit |
| `virtual-users portfolio` | 1 credit/address |
| `approvals` | 1 credit |
| `airdrop` | 1 credit |
| `polymarket` | 1 credit |
| `agent wallet` | x402 payment |
| `agent portfolio` | x402 payment |
| `agent nav` | x402 payment |
| `agent status` | x402 payment |
| `agent chains` | x402 payment |

Purchase credits at [octav.fi](https://octav.fi).

## Supported Chains

Ethereum, Solana, Arbitrum, Base, Polygon, Optimism, BNB Chain, Avalanche, Fantom, Cronos, Gnosis, Celo, Moonbeam, Moonriver, Harmony, Aurora, Metis, Boba, Fuse, Evmos, Kava, and more.

## License

MIT

## Links

- [Octav Website](https://octav.fi)
- [Octav API Docs](https://docs.octav.fi)
- [Octav MCP Server](https://github.com/Octav-Labs/octav-api-mcp)
