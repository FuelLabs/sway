# Testing with `forc-node`

`forc-node` wraps the `fuel-core` library and provides a convenient CLI for starting a Fuel node.  
Besides running an entirely local chain, `forc-node` can *fork* an existing node - or network (testnet or mainnet) so that you can exercise contracts against a near-real state without touching live infrastructure.

The workflow below demonstrates how to validate contract reads against public Fuel Testnet data and then repeat the exact same call against a forked local node.

## 1. Inspect a contract on testnet

Fuel ships a `forc-call` binary that can read contract state directly from public infrastructure.  
The example below queries the owner of the [Mira AMM](https://github.com/mira-amm/mira-v1-periphery) contract that is already deployed on testnet:

```sh
forc-call \
  --abi https://raw.githubusercontent.com/mira-amm/mira-v1-periphery/refs/heads/main/fixtures/mira-amm/mira_amm_contract-abi.json \
  0xd5a716d967a9137222219657d7877bd8c79c64e1edb5de9f2901c98ebe74da80 \
  owner \
  --testnet
```

Sample output (truncated):

```
…
result: Initialized(Address(std::address::Address { bits: Bits256([31, 131, 36, 111, 177, 67, 191, 23, 136, 60, 86, 168, 69, 88, 194, 77, 47, 157, 117, 51, 25, 181, 34, 234, 129, 216, 182, 250, 160, 158, 176, 83]) }))
```

Keep both the ABI URL and the contract ID handy—we will reuse them when pointing `forc-node` at the same network.

## 2. Start a forked node that mirrors testnet

Launch a local `forc-node` instance and instruct it to sync contract state from the public Testnet GraphQL endpoint.  
The first time a contract or storage slot is requested, the forked node lazily retrieves the data from the remote network and caches it into the local database.

```sh
cargo run -p forc-node -- \
  local \
  --fork-url https://testnet.fuel.network/v1/graphql \
  --db-type rocks-db \
  --db-path /tmp/.db.fork \
  --debug \
  --historical-execution \
  --poa-instant \
  --port 4000
```

Key flags:

- `--fork-url` specifies the upstream GraphQL endpoint; for Testnet this is `https://testnet.fuel.network/v1/graphql`.
- `--db-type rocks-db` and `--db-path` enable persistence so the node survives restarts; this is required for `historical-execution` to work.
- `--historical-execution` allows dry-running transactions against previous blocks—handy when replaying test cases.
  - This is required for state forking to work.
- `--poa-instant` auto-produces blocks so transactions submitted against the fork finalize immediately.

Once the node is running, its GraphQL endpoint is available at `http://127.0.0.1:4000/v1/graphql`.

## 3. Repeat the contract call against the fork

Now that the forked node is live, repeat the earlier `forc-call` but target the local endpoint instead of the public Testnet endpoint:

```sh
forc-call \
  --abi https://raw.githubusercontent.com/mira-amm/mira-v1-periphery/refs/heads/main/fixtures/mira-amm/mira_amm_contract-abi.json \
  0xd5a716d967a9137222219657d7877bd8c79c64e1edb5de9f2901c98ebe74da80 \
  owner \
  --node-url http://127.0.0.1:4000/v1/graphql
```

The first call hydrates the contract bytecode and storage into the RocksDB database defined earlier; subsequent reads are served locally.  
You should see the same owner address as before:

```
…
result: Initialized(Address(std::address::Address { bits: Bits256([31, 131, 36, 111, 177, 67, 191, 23, 136, 60, 86, 168, 69, 88, 194, 77, 47, 157, 117, 51, 25, 181, 34, 234, 129, 216, 182, 250, 160, 158, 176, 83]) }))
```

Because the fork persists data locally, re-running the command now serves the response immediately without contacting Testnet again.  
You can safely mutate state or deploy additional tooling against the fork—the remote network remains untouched.

### Verifying fork behaviour

- **Lazy hydration:** the first query fetches bytecode and storage from Testnet; later calls are local.
- **State isolation:** writes against the fork do not propagate back to Testnet.
- **Continued discovery:** if you reference another contract that exists on Testnet, the fork loads it on demand, letting you blend public and local-only workflows.

## Troubleshooting and tips

The original feature proposal and design discussion is tracked in [FuelLabs/sway#7448](https://github.com/FuelLabs/sway/issues/7448) if you need more background.
