# relay-cli

CLI for the [Relay](https://relay.link) cross-chain bridge/swap protocol. Bridge and swap assets across 85+ chains from your terminal.

## Install

### From source

```sh
cargo install --path .
```

### Pre-built binary

Download from [releases](https://github.com/saurabh/relay-cli/releases) and put `relay` in your `$PATH`.

## Setup

```sh
# Set your wallet private key
export RELAY_PRIVATE_KEY=0x...

# Optional: API key for higher rate limits
export RELAY_API_KEY=your-key

# Or persist via config
relay config set --api-key your-key
```

Config is stored at `~/.relay/config.json`.

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
relay price 0xa5D0016B11AA203a25fE39E548573DdFB0e77702 --chain 1
```

### `relay quote`

Get a quote for bridging or swapping without executing.

```sh
relay quote \
  --from-chain 1 --from-currency ETH \
  --to-chain 8453 --to-currency ETH \
  --amount 1000000000000000000 \
  --user 0xYourAddress
```

Token symbols (`ETH`, `USDC`, `WBTC`, etc.) are resolved automatically. Use `--amount` in wei.

### `relay bridge`

Quote and execute a cross-chain bridge/swap.

```sh
relay bridge \
  --from-chain 1 --from-currency ETH \
  --to-chain 8453 --to-currency ETH \
  --amount 1000000000000000000 \
  --user 0xYourAddress

# To a different recipient
relay bridge ... --recipient 0xOtherAddress
```

Requires `RELAY_PRIVATE_KEY`. Each chain needs an RPC URL set via env:

```sh
export RPC_1=https://eth-mainnet.g.alchemy.com/v2/your-key
export RPC_8453=https://base-mainnet.g.alchemy.com/v2/your-key
```

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
relay config set --api-key your-key
relay config set --testnet true
```

## Global flags

| Flag            | Env                 | Description        |
| --------------- | ------------------- | ------------------ |
| `--api-key`     | `RELAY_API_KEY`     | Relay API key      |
| `--private-key` | `RELAY_PRIVATE_KEY` | Wallet private key |
| `--testnet`     | —                   | Use testnet API    |

## RPC URLs

Each chain you transact on needs an RPC URL. Set via env var `RPC_<chainId>`:

```sh
export RPC_1=https://...       # Ethereum
export RPC_8453=https://...    # Base
export RPC_42161=https://...   # Arbitrum
export RPC_10=https://...      # Optimism
```

## License

MIT
