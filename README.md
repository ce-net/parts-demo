# parts.demo

Parts inventory demo of the ce-db typed collections: a Rust CLI and a tailnet page on state.store that update each other live. One function, one cell, one repo (ce-fleet standards/a-cell-is-one-function.md).

- API: `parts-demo-api` (`parts.demo` v1): `HealthOp` (needs `parts.demo:health`), `Run` (needs `parts.demo:run`), and the shared `CellSpawn/CellInstances/CellStop/CellDeploy/CellBench` (900-904, needs `parts.demo:spawn|instances|stop|deploy|bench`).
- Depends on: none.
- Start: the door reads [cell.toml](cell.toml); `parts-demo serve` registers with this machine's door. Nobody starts it by hand: a caller's `by_name` finds it over net.find or starts it.
- Clients: Rust `ce_wire::Client::<parts_demo_api::PartsDemo>::by_name(token)`; TypeScript `sdk/ts/PartsDemo.ts`; browser `sdk/ts-web/PartsDemo.ts`; Swift `sdk/swift/PartsDemo.swift` (CI regenerates all with ce-wire-gen and fails on drift). Keep a connection up with `Link` (the same states in every language: idle, finding, live, stale, reconnecting, refused, failed).
- Changing the api: an op's request or answer may not change shape at the same version. `ce-wire-gen freeze crates/parts-demo-api/src/lib.rs` keeps v1 in `src/versions/`, then change the v2 block and run `ce-wire-gen adapt ... --from src/versions/v1.rs`: old callers keep working through the adapters, and new callers reach old servers with `Client::by_name_older` (rust-stack docs/SCHEMA-ADAPTERS.md). `ce-wire-gen check` (in CI) fails an api change without the freeze.
- Serves many callers: every call is scoped to the caller's key (`Caller.name`), state and limits are per caller, settings per deployment (`PARTS_DEMO_CALLS_PER_MIN`, `PARTS_DEMO_MAX_OWNERS`); `tests/many.rs` checks it (ce-fleet standards/cells-serve-many.md).
- Test: `cargo test`.

Made by cell.forge `new-cell parts.demo`.
