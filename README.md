# Rain USDR — Soroban Port

A Rust / [Soroban](https://developers.stellar.org/docs/build/smart-contracts/overview) re-implementation of the
**USDR (Rain Dollar)** protocol: a multi-collateral, over-collateralized stablecoin originally written in Solidity
and deployed on Arbitrum One.

This repository is a **1:1 port**, not a rewrite. The module boundaries, the accounting invariants, the security
checks and even the comment-level reasoning are carried over from the original Solidity source, so that a
reviewer can diff the two side by side. Where Soroban forces a different shape, the difference is called out
explicitly in the code and listed below.

> **Not audited.** Like the original, this is an engineering implementation of the USDR specification, not audited
> code. Do not deploy it to mainnet.

---

## The system

USDR targets one US dollar and is backed by a diversified basket of collateral (USDT, USDC and RAIN). The design
is a direct adaptation of MakerDAO's Multi-Collateral Dai, enforcing a single rule at every point in time:

> **the protocol's maximum possible loss can never exceed its stable reserves.**

The **immutable core** (the Vault Engine, the liquidation logic and the solvency rule) cannot change after
deployment. Only risk parameters can be tuned, and only through the Governor's timelock.

## Contract map

Each Solidity contract becomes its own crate so it can be deployed independently.

| Solidity (`contracts/`) | Soroban crate | MakerDAO analogue | Status |
| --- | --- | --- | --- |
| `token/USDR.sol` | `contracts/usdr` | Dai | ✅ ported + tested |
| `core/VaultEngine.sol` | `contracts/vault-engine` | Vat | ✅ ported + tested |
| `oracle/PriceConverter.sol` | `contracts/price-converter` | Spot | ✅ ported |
| `reserve/ReserveAccounting.sol` | `contracts/reserve-accounting` | (custom) | ✅ ported + tested |
| `liquidation/PriceCurve.sol` | `contracts/price-curve` | Abacus | ✅ ported + tested |
| `shared/*`, `libraries/Math.sol` | `contracts/shared` | — | ✅ ported |
| `interfaces/*.sol` | `contracts/interfaces` | — | ✅ ported (traits + clients) |
| `core/CollateralAdapter.sol` | `contracts/collateral-adapter` | GemJoin + DaiJoin | ⬜ planned |
| `oracle/OracleSecurityModule.sol` | `contracts/oracle-security-module` | OSM | ⬜ planned |
| `reserve/SolvencyEngine.sol` | `contracts/solvency-engine` | (custom) | ⬜ planned |
| `reserve/BalanceSheet.sol` | `contracts/balance-sheet` | Vow | ⬜ planned |
| `reserve/PegStabilityModule.sol` | `contracts/peg-stability-module` | PSM | ⬜ planned |
| `liquidation/LiquidationTrigger.sol` | `contracts/liquidation-trigger` | Dog | ⬜ planned |
| `liquidation/DutchAuction.sol` | `contracts/dutch-auction` | Clipper | ⬜ planned |
| `liquidation/CircuitBreaker.sol` | `contracts/circuit-breaker` | (custom) | ⬜ planned |
| `governance/Governor.sol` | `contracts/governor` | Spell + Pause | ⬜ planned |
| `governance/End.sol` | `contracts/end` | End | ⬜ planned |

All interfaces, events, storage structs and error variants from the original are already ported, so the
remaining contracts only need their logic written.

## Repository layout

```
contracts/
  shared/        constants (WAD/RAY/RAD, roles), errors, events, fixed-point math, types, storage + role helpers
  interfaces/    one module per Solidity interface, as `#[contractclient]` traits that generate typed clients
  usdr/          …
  vault-engine/
  …
```

## Soroban adaptations

These are the non-mechanical differences between the two implementations. They are the things to check first
when reviewing a ported contract.

| Solidity / EVM | Soroban / Stellar | Why |
| --- | --- | --- |
| `uint256` / `int256` | `I256` (signed 256-bit) | Soroban's widest native integer is `i128`, which cannot hold `_RAD` (10^45). `I256` preserves MakerDAO's exact decimal semantics. All ledger values are non-negative and far below 2^255. |
| `bytes32 ilkId` | `BytesN<32>` built by `shared::constants::ilk_id` | `bytes32` has no Soroban equivalent beyond `BytesN`; identifiers stay byte-identical to the EVM deployment so indexers can migrate. |
| `keccak256("WARD_ROLE")` | `Symbol` (`WARD`, `BURNER`, `COMMITTER`, `READER`, `RECORDER`) | Roles are behaviour, not hashes; short symbols are cheaper and readable. Grant/revoke/admin semantics are unchanged. |
| `msg.sender` | an explicit `sender: Address` argument with `require_auth()` | Soroban exposes no ambient caller. Contract-to-contract calls pass `env.current_contract_address()`, which a direct sub-invocation authorizes automatically. |
| `onlyRole(_WARD_ROLE)` | `caller.require_auth()` + `access::require_role` | Same gate, expressed in the Soroban authorization model. |
| `address(0)` as "unset" | `Option<Address>` | Soroban has no zero address. |
| overloaded `file(bytes32,uint256)` / `file(bytes32,address)` | `file(what, FileValue)` and `file_ilk(ilk_id, what, FileValue)` | Rust traits and the Soroban ABI cannot express overloads. `FileValue::Num` is the `uint256` overload, `FileValue::Addr` the `address` overload. |
| `move(from,to,rad)` | `move_rad` | `move` is a Rust keyword. |
| tuple getters, e.g. `ilks(id) → (6 values)` | a `#[contracttype]` struct (`shared::types::Ilk`) | Structs are self-describing and shared by implementations and clients. |
| `event File(bytes32,uint256)` and friends | separate `FileNum` / `FileAddr` / `FileIlk` events | Distinct names keep them in the contract spec without collision. |
| `emit` / `event` | `#[contractevent]` structs with `.publish(&env)` | The SDK's current idiom; also registers the event in the contract spec for indexers. |
| `bytes calldata` in the Governor | `Symbol` fn name + `Vec<Val>` args, dispatched with `invoke_contract` | Soroban has no EVM-style calldata. |
| `mapping` | typed `DataKey` enums + `persistent()` / `instance()` storage | Explicit storage layout. |
| (no concept) | TTL extension on every persistent write | **New behaviour, not a translation.** Soroban persistent entries expire; `shared::storage::set` extends the TTL to ~29 days. Dropping this silently loses ledger state. |
| EIP-2612 `permit` | omitted | Soroban handles signatures through the host's authorization entries. |
| `ReentrancyGuard` | not needed | Soroban contracts cannot be re-entered across frames; host reentrancy is rejected. |

### Unit conventions (unchanged)

* `wad` — 18 decimals, token quantities
* `ray` — 27 decimals, rates and price factors
* `rad` — 45 decimals, internal USDR balances

Note that `line` and `globalLine` are **rad**, while `spot` and `rate` are **ray**. Mixing these up is the most
common porting mistake; `vault-engine`'s tests exercise both ceilings deliberately.

## Getting set up

Requires the Rust toolchain with the `wasm32v1-none` target (pinned in `rust-toolchain.toml`) and the Stellar
CLI for deployment tooling.

```bash
git clone <this repo>
cd rain-usdr-soroban
cargo test --workspace                              # run the test suite
cargo build --target wasm32v1-none --release        # build deployable WASM
stellar contract build --workspace                  # optimized build via the CLI
```

## Testing

Solidity's Foundry suite becomes Soroban unit tests that run against the in-process host:

```rust
#[test]
fn frob_locks_collateral_and_draws_debt_within_the_ratio() {
    let env = Env::default();
    let (admin, engine, rain) = setup(&env);
    // ...
}
```

`env.mock_all_auths()` stands in for the test harness's `vm.prank`. Cross-contract flows are exercised by
deploying the real contracts and wiring them together, exactly as the Foundry base harness does.

## Contributing

This project exists to be contributed to.

1. Pick a contract from the **planned** list above and open an issue so the interface wiring can be agreed.
2. Port it in its own crate, following the structure of an already-ported contract (`vault-engine` is the
   reference implementation: it covers roles, cross-contract calls, the `I256` math helpers and events).
3. Port the corresponding Foundry assertions from the original `tests/` directory into `src/test.rs`.
4. Keep the original comments. The reasoning in the Solidity comments is the specification; adapt it rather than
   deleting it.
5. Run `cargo test --workspace` and `cargo clippy --workspace` before opening a PR.

Every deviation from the Solidity original must be justified in a comment at the point of deviation, and listed
in the table above if it is systemic.# -predictionMarketSoroban
