---
name: relay-cli
description: Use when working with relay-cli — a Rust CLI for cross-chain bridging/swapping via the Relay protocol. Covers installation, config setup, all commands, token/amount formats, RPC configuration, and execution flow.
---

# relay-cli

Rust CLI for the [Relay](https://relay.link) cross-chain bridge/swap protocol. Supports 85+ EVM chains.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/starc007/relay-cli/main/install.sh | sh
```

Or build from source:
```bash
git clone https://github.com/starc007/relay-cli
cd relay-cli
cargo build --release
```

Binary: `relay`

## Config

Config lives at `~/.relay/config.json`. Auto-created on first run with default RPCs for 11 chains.

### Set credentials

```bash
relay config set --api-key YOUR_KEY          # Relay API key (optional for quotes)
relay config set --private-key 0xYOUR_PK     # EVM private key (required for bridge)
relay config set --testnet true              # Switch to testnet
```

### View config (PK redacted)

```bash
relay config show
```

### RPC management

```bash
relay config list-rpcs                       # Show all configured RPCs
relay config set-rpc --chain 1 --url https://eth.llamarpc.com
```

Default RPCs (auto-configured):

| Chain ID | Network |
|----------|---------|
| 1 | Ethereum |
| 8453 | Base |
| 42161 | Arbitrum |
| 10 | Optimism |
| 137 | Polygon |
| 43114 | Avalanche |
| 56 | BSC |
| 324 | zkSync Era |
| 534352 | Scroll |
| 59144 | Linea |
| 7777777 | Zora |

### Env var overrides

```bash
RELAY_API_KEY=...        # same as --api-key
RELAY_PRIVATE_KEY=...    # same as --private-key
RELAY_WALLET=...         # wallet address for quote/bridge/history
RPC_<CHAIN_ID>=...       # override RPC for specific chain (e.g. RPC_1=https://...)
```

## Token Formats

`--from-currency` and `--to-currency` accept three formats:

| Format | Example | Notes |
|--------|---------|-------|
| 0x address | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` | Exact contract |
| Native shorthand | `ETH`, `MATIC`, `BNB`, `AVAX`, `ARB`, `OP` | Resolved to zero address |
| Symbol | `USDC`, `WETH` | Looked up via Relay currencies API |

## Amount Format

`--amount` accepts human-readable decimals. Automatically converted to wei using token decimals.

```
0.001       →  1000000000000000   (ETH, 18 decimals)
1.5         →  1500000            (USDC, 6 decimals)
100         →  100000000000000000000
```

## Commands

### `relay chains`

List supported chains.

```bash
relay chains
relay chains --filter base     # filter by name
```

### `relay tokens`

List tokens on a chain.

```bash
relay tokens --chain 8453
relay tokens --chain 8453 --filter usdc
relay tokens --chain 8453 --verified    # verified tokens only
```

### `relay price`

Get USD price of a token.

```bash
relay price ETH --chain 1
relay price USDC --chain 8453
relay price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain 1
```

### `relay quote`

Get a bridge/swap quote without executing.

```bash
relay quote \
  --from-chain 1 \
  --from-currency ETH \
  --to-chain 8453 \
  --to-currency ETH \
  --amount 0.01 \
  --user 0xYOUR_WALLET
```

Output: send amount, receive amount, ETA in seconds.

Optional: `--recipient 0xOTHER` to send to a different address.

### `relay bridge`

Get quote then execute the bridge/swap. Requires private key.

```bash
relay bridge \
  --from-chain 1 \
  --from-currency ETH \
  --to-chain 8453 \
  --to-currency ETH \
  --amount 0.01 \
  --user 0xYOUR_WALLET

# With explicit private key
relay bridge ... --private-key 0xYOUR_PK

# Cross-token swap (ETH → USDC)
relay bridge \
  --from-chain 1 --from-currency ETH \
  --to-chain 8453 --to-currency USDC \
  --amount 0.1 --user 0xYOUR_WALLET
```

Execution flow:
1. Resolves token symbols → contract addresses + decimals
2. Converts human amount → wei
3. Fetches quote from Relay API (`/quote`)
4. Prints quote summary
5. Executes steps from quote:
   - `Transaction` steps: signs and sends via alloy provider using configured RPC
   - `Signature` steps: signs EIP-191 or EIP-712 data, posts to step endpoint
6. Polls `/intents/status/v2` until confirmed or failed

### `relay status`

Check status of a bridge request.

```bash
relay status REQUEST_ID
relay status REQUEST_ID --watch    # poll until terminal state
```

Terminal states: `success`, `failure`, `refund`

### `relay history`

View past bridge transactions for a wallet.

```bash
relay history --user 0xYOUR_WALLET
relay history --user 0xYOUR_WALLET --limit 50
relay history --user 0xYOUR_WALLET --status success
# status options: success, failure, refund, pending, depositing
```

### `relay update`

Self-update to latest GitHub release.

```bash
relay update
```

Detects platform (macOS/Linux × x86_64/aarch64), downloads tarball from GitHub releases, replaces running binary.

### `relay config`

```bash
relay config show
relay config set --api-key KEY --private-key 0xPK --testnet false
relay config set-rpc --chain 1 --url https://eth.llamarpc.com
relay config list-rpcs
```

## Global Flags

These work with any subcommand:

```
--api-key KEY        Relay API key
--private-key 0xPK   EVM private key (hidden from logs)
--testnet            Use testnet endpoints
```

## API Endpoints Used

| Purpose | Endpoint |
|---------|----------|
| Chains | `GET /chains` |
| Tokens | `GET /currencies/v1?chainId=...` |
| Token price | `GET /currencies/token/price?address=...&chainId=...` |
| Quote | `POST /quote` |
| Status | `GET /intents/status/v2?requestId=...` |
| History | `GET /requests/v2?user=...&limit=...` |

Base URLs:
- Mainnet: `https://api.relay.link`
- Testnet: `https://api.testnets.relay.link`

## Common Patterns for AI Agents

**Bridge ETH from Ethereum to Base:**
```bash
relay bridge --from-chain 1 --from-currency ETH --to-chain 8453 --to-currency ETH --amount 0.05 --user 0xADDR
```

**Swap ETH to USDC on Base:**
```bash
relay bridge --from-chain 8453 --from-currency ETH --to-chain 8453 --to-currency USDC --amount 0.1 --user 0xADDR
```

**Check if a bridge completed:**
```bash
relay status REQUEST_ID --watch
```

**Find chain IDs:**
```bash
relay chains --filter ethereum   # → chain ID 1
relay chains --filter base       # → chain ID 8453
```

**Find token address:**
```bash
relay tokens --chain 1 --filter usdc --verified
```
