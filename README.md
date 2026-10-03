# PayRaider Contracts

**Soroban smart contracts that anchor PayRaider's analytics on the Stellar ledger, so published numbers can be verified rather than trusted.**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
![Soroban](https://img.shields.io/badge/Stellar-Soroban-black)
![Rust](https://img.shields.io/badge/Rust-no__std-orange)

Part of [PayRaider](https://github.com/Pay-Raider): [backend](https://github.com/Pay-Raider/payraider-backend) · [web app](https://github.com/Pay-Raider/payraider-app) · [plugin & SDKs](https://github.com/Pay-Raider/payraider-plugin) · [mobile](https://github.com/Pay-Raider/payraider-mobile)

---

## Why on-chain

PayRaider tells payout apps whether a corridor is healthy. Those numbers matter, so anyone should be able to check they were not changed after the fact. For each analytics epoch, the backend hashes its snapshot (SHA-256) and records the hash in the `payraider` contract. Anyone can recompute the hash from the published snapshot and compare it with the ledger.

## The `payraider` contract

| Function | What it does |
| --- | --- |
| `initialize(admin)` | One-time setup |
| `submit_snapshot(epoch, hash, caller)` | Record a snapshot hash for an epoch. Epochs must increase; duplicates are rejected |
| `get_snapshot(epoch)` | The hash recorded for an epoch |
| `latest_snapshot()` | Latest hash, epoch and ledger timestamp |
| `get_latest_epoch()` | Latest epoch number |
| `pause` / `unpause` | Stop or resume submissions (admin) |
| `set_admin`, `set_governance` | Hand over control |
| `upgrade`, `migrate` | Upgrade the contract code and storage |

Events are emitted for submissions and admin changes; see [`payraider/src/events.rs`](payraider/src/events.rs).

### Deployed (testnet)

| Contract | ID |
| --- | --- |
| `payraider` | `CAPHQZ4BBT43HU5EUSJAOPKWB66HGLTN4AKJUALV3R2RXS4A6IOXWUTL` |

[`.env.testnet`](.env.testnet) is written by the deploy script and lists every deployed ID.

## Quick start

Requires Rust with the `wasm32v1-none` target and the [Stellar CLI](https://developers.stellar.org/docs/tools/cli).

```bash
rustup target add wasm32v1-none
cargo test                                   # unit and integration tests
stellar contract build                       # build the WASM
```

Deploy to testnet:

```bash
./scripts/deploy-contracts-testnet.sh
```

Mainnet deployment has a checklist and verification step: [`scripts/pre-mainnet-checklist.sh`](scripts/pre-mainnet-checklist.sh), [`scripts/deploy-contracts-mainnet.sh`](scripts/deploy-contracts-mainnet.sh), [`scripts/verify-contract-mainnet.sh`](scripts/verify-contract-mainnet.sh).

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -W clippy::all
cargo test
./scripts/run-contract-fuzz.sh               # property-based fuzzing
```

Gas costs are tracked in [`docs/GAS_COSTS.md`](docs/GAS_COSTS.md), and `scripts/check_gas_regression.py` fails on regressions.

## Project layout

| Path | Contents |
| --- | --- |
| `payraider/` | The production contract, used by the backend (`SNAPSHOT_CONTRACT_ID`) |
| `tests/` | Integration and fuzz tests |
| `benches/` | Gas benchmarks |
| `scripts/` | Deploy, verify and fuzz scripts |
| `access-control/`, `analytics/`, `escrow/`, … | Earlier contract experiments, kept for reference and excluded from the workspace (see [`archive/README.md`](archive/README.md)) |

## License

[Apache 2.0](LICENSE)
