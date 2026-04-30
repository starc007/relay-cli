# relay-cli

CLI for the [Relay](https://relay.link) cross-chain bridge/swap protocol. Bridge and swap assets across 85+ chains from your terminal.

## Install

### One-liner (macOS / Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/saurabh/relay-cli/main/install.sh | sh
```

Detects your OS and architecture, downloads the right binary, installs to `/usr/local/bin/relay`.

### From source

```sh
cargo install --path .
```

### Manual

Download a binary from [releases](https://github.com/saurabh/relay-cli/releases), extract, and put `relay` in your `$PATH`.

## Setup

```sh
relay config set --private-key 0x...
relay config set --api-key your-key   # optional, for higher rate limits
```

Config is stored at `~/.relay/config.json`. Private key is masked in `relay config show`.

RPC URLs for 11 major EVM chains are bundled by default (Ethereum, Base, Arbitrum, Optimism, Polygon, Avalanche, BSC, zkSync, Scroll, Linea, Zora). No extra setup needed to bridge on these chains.

## Commands

### `relay chains`

List all supported chains.

```sh
relay chains
relay chains --filter base
relay chains --filter 8453
```

### `relay tokens`

List tokens available on a chain.

```sh
relay tokens --chain 8453
relay tokens --chain 1 --filter usdc
relay tokens --chain 8453 --verified
```

### `relay price`

Get USD price for a token.

```sh
relay price ETH --chain 1
relay price USDC --chain 8453
relay price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain 1
```

### `relay quote`

Get a quote without executing. `--amount` is in human-readable units.

```sh
relay quote \
  --from-chain 1 --from-currency ETH \
  --to-chain 8453 --to-currency ETH \
  --amount 0.001 \
  --user 0xYourAddress

relay quote \
  --from-chain 1 --from-currency USDC \
  --to-chain 42161 --to-currency USDC \
  --amount 100 \
  --user 0xYourAddress
```

Token symbols (`ETH`, `USDC`, `WBTC`, etc.) resolve to contract addresses automatically.

### `relay bridge`

Quote and execute a cross-chain bridge/swap.

```sh
relay bridge \
  --from-chain 1 --from-currency ETH \
  --to-chain 8453 --to-currency ETH \
  --amount 0.001 \
  --user 0xYourAddress

# Different recipient
relay bridge \
  --from-chain 1 --from-currency ETH \
  --to-chain 8453 --to-currency ETH \
  --amount 0.001 \
  --user 0xYourAddress \
  --recipient 0xOtherAddress
```

Requires a private key set via `relay config set --private-key` or `RELAY_PRIVATE_KEY` env var.

### `relay status`

Check the status of a bridge request.

```sh
# One-shot
relay status 0xrequestId...

# Poll until complete
relay status 0xrequestId... --watch
```

Status values: `pending`, `success`, `failure`, `refund`.

### `relay history`

View past transactions for a wallet.

```sh
relay history --user 0xYourAddress
relay history --user 0xYourAddress --limit 50
relay history --user 0xYourAddress --status success
```

Status filter options: `success`, `failure`, `refund`, `pending`, `depositing`.

### `relay config`

```sh
relay config show
relay config set --private-key 0x...
relay config set --api-key your-key
relay config set --testnet true

# RPC management
relay config list-rpcs
relay config set-rpc --chain 1 --url https://eth-mainnet.g.alchemy.com/v2/key
```

## Global flags

| Flag            | Env                 | Description                      |
| --------------- | ------------------- | -------------------------------- |
| `--api-key`     | `RELAY_API_KEY`     | Relay API key (higher rate limit) |
| `--private-key` | `RELAY_PRIVATE_KEY` | Wallet private key               |
| `--testnet`     | —                   | Use testnet API                  |

## RPC URLs

Default public RPCs are bundled for these chains:

| Chain ID | Network   |
| -------- | --------- |
| 1        | Ethereum  |
| 10       | Optimism  |
| 56       | BSC       |
| 137      | Polygon   |
| 324      | zkSync    |
| 8453     | Base      |
| 42161    | Arbitrum  |
| 43114    | Avalanche |
| 59144    | Linea     |
| 534352   | Scroll    |
| 7777777  | Zora      |

Override any or add others:

```sh
relay config set-rpc --chain 1 --url https://eth-mainnet.g.alchemy.com/v2/key
relay config set-rpc --chain 81457 --url https://rpc.blast.io
```

Env var `RPC_<chainId>` takes priority over config if both are set.

## License

MIT
