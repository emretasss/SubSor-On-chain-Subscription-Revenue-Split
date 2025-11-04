# SubSor — On-chain Subscription & Revenue Split

**Repository:** `stellarebootcampproject`

A hackathon-ready Soroban (Stellar) smart contract for subscription management and automated revenue splitting.

---

## TL;DR

* Contract implemented in Rust using Soroban SDK (`contract/src/lib.rs`).
* Build → (optional) optimize → deploy to Futurenet/local sandbox → invoke.
* Generate TypeScript bindings for a React frontend (Freighter-compatible).

---

## Features

* Create, cancel, renew subscriptions (`create_subscription`, `cancel_subscription`, `renew_subscription`).
* Automated revenue splitting and recipient balance tracking (`withdraw_revenue`, `get_balance`).
* Paginated queries and owner subscription lists (`list_subscriptions`, `get_all_subscriptions`).
* Safe arithmetic (checked operations) and Soroban storage patterns.

---

## Quick Requirements

* Rust toolchain (rustup)
* WASM target: `wasm32-unknown-unknown`
* `soroban` or `stellar` CLI (soroban-cli / stellar-cli)
* `wasm-opt` (Binaryen) — recommended for optimization
* Node.js + npm/yarn for frontend

---

## Setup

```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# (Optional) Install Binaryen for wasm-opt
# Windows examples: scoop install binaryen  OR  choco install binaryen

# Install soroban/stellar CLI (follow official docs)
# Example: cargo install soroban-cli
```

---

## Build (contract)

```bash
cd contract
cargo build --release --target wasm32-unknown-unknown
# artifact: target/wasm32-unknown-unknown/release/subsor.wasm
```

> Note: If build succeeds but optimize fails with bulk-memory validation errors, either update Binaryen or skip optimization and deploy the unoptimized WASM.

---

## Optimize (recommended)

```bash
# Using wasm-opt (Binaryen)
wasm-opt -Oz target/wasm32-unknown-unknown/release/subsor.wasm -o target/wasm32-unknown-unknown/release/subsor.optimized.wasm

# Or use CLI wrapper if available
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/subsor.wasm
```

---

## Network notes (Futurenet vs Testnet)

* Soroban smart contracts run on **Futurenet** or a local Soroban sandbox. The classic Stellar Testnet does not support Soroban contracts.
* Some CLI configs use a `testnet` alias that points to Futurenet RPC. Verify with:

```bash
soroban config network show testnet
```

---

## Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/subsor.wasm \
  --source-account <YOUR_ALIAS> \
  --network testnet

# or deploy optimized wasm if available
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/subsor.optimized.wasm \
  --source-account <YOUR_ALIAS> \
  --network testnet
```

* If you see `Account not found`, fund your alias with Friendbot or create an identity: `soroban config identity generate <name>` and fund it.
* Deploy output returns the **Contract ID** (e.g. `CAWL...`). Save it.

---

## Initialize (if contract exposes `initialize`)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <YOUR_ALIAS> \
  --network testnet \
  -- \
  initialize
```

If `initialize` traps with `UnreachableCodeReached` or `InvalidAction`, check the function's required authorization (`require_auth`) and call it from the correct account.

---

## Common invocations (examples)

Replace `<CONTRACT_ID>` and `<ADDR>` placeholders with real values.

### Create subscription (owner calls or owner-authorized caller)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <CALLER_ALIAS> \
  --network testnet \
  -- \
  create_subscription \
  --owner <OWNER_ADDRESS> \
  --subscriber <SUBSCRIBER_ADDRESS> \
  --amount 1000000 \
  --period_days 30 \
  --recipient <RECIPIENT_ADDRESS> \
  --split_percentage 1500
```

* `amount` is in stroops (1 XLM = 10,000,000 stroops); `1000000` = 0.1 XLM.
* `split_percentage` is in basis points (1500 = 15%).
* If you get `Missing signing key`, set `--source-account` to an alias that has the private key registered with the CLI (e.g. `alice2`).

### Read subscription

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <ANY_ALIAS> \
  --network testnet \
  -- \
  get_subscription \
  --subscription_id 1
```

### Renew / process due subscriptions

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <CALLER_ALIAS> \
  --network testnet \
  -- \
  renew_subscription \
  --subscription_id 1

# Or process batch for owner
stellar contract invoke --id <CONTRACT_ID> --source-account <OWNER_ALIAS> --network testnet -- process_due_subscriptions --owner <OWNER_ADDRESS> --max_count 10
```

### Withdraw revenue

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <RECIPIENT_ALIAS> \
  --network testnet \
  -- \
  withdraw_revenue \
  --recipient <RECIPIENT_ADDRESS>
```

---

## Create test accounts (example)

```bash
stellar keys generate alice2 --network testnet --fund
stellar keys generate bob --network testnet --fund
stellar keys generate merchant --network testnet --fund

stellar keys address alice2
stellar keys address bob
stellar keys address merchant
```

---

## TypeScript bindings (generate)

```bash
soroban contract bindings typescript \
  --wasm target/wasm32-unknown-unknown/release/subsor.wasm \
  --contract-id <CONTRACT_ID> \
  --output-dir ../bindings
```

Import the generated bindings in the frontend for type-safe contract calls.

---

## Frontend (React) quickstart

```bash
cd frontend
npm install
# configure CONTRACT_ID and RPC in src/config or env
npm start
```

Frontend responsibilities:

* Wallet connection with Freighter
* Call bindings for create/renew/withdraw
* Display balances and subscription lists

---

## Tests & CI

```bash
# Rust unit tests
cd contract
cargo test

# E2E script example (sandbox/Futurenet)
# ./scripts/e2e.sh <CONTRACT_ID> <ALICE_ALIAS> <BOB_ADDRESS> <MERCHANT_ADDRESS>
```

---

## Troubleshooting

* **`Account not found`**: Fund the alias or import a secret key. Use Friendbot for testnet/futurenet aliases.
* **`Missing signing key`**: The CLI does not have the private key for the `--source-account` alias. Import secret or use an alias you generated with `stellar keys generate`.
* **WASM optimize errors (bulk memory)**: Install or update Binaryen (`wasm-opt`) or run optimize in WSL/Linux. You can also skip optimization and deploy the raw `.wasm`.
* **`UnreachableCodeReached` / `InvalidAction`**: Check authorization and input validation in the contract. Use the correct signer.

---

## Project structure

```
stellarebootcampproject/
├── contract/
│   ├── src/
│   │   └── lib.rs        # contract implementation
│   └── Cargo.toml
├── frontend/
├── bindings/             # generated TypeScript bindings
├── scripts/
└── README.md
```

---

## Notes & Recommendations

* Prefer `crate-type = ["cdylib"]` in Cargo.toml to avoid native link errors on Windows.
* Lock Soroban SDK to a specific compatible version (e.g. `22.0.8`) in Cargo.toml.
* On Windows, building/optimizing can be flaky; WSL (Ubuntu) is often more reliable for wasm-opt.

---

## License

MIT


