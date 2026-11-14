# parts.demo

The slice 1 demo of the ce-db typed collections (`repo:state-store-cell/docs/DB-SDK-DESIGN.md` section 9): a parts
inventory in state.store, app `parts-demo`, collection `parts`, used from a Rust CLI and from a page this cell serves.
A change made in either appears in the other live. One function, one cell, one repo.

## One schema, three outputs

`crates/parts-schema/src/lib.rs` is the only definition of a part:

```rust
#[derive(Record, Clone, Debug, PartialEq)]
#[db(app = "parts-demo", collection = "parts", version = 2)]
pub struct Part {
    #[db(key)] pub id: Id,
    #[db(max = 120)] pub name: String,
    #[db(index, max = 16)] pub bin: String,
    #[db(index, ordered)] pub qty: u32,
    #[db(since = 2, max = 80)] pub supplier: Option<String>,
    #[db(since = 2, max = 16)] pub by: Option<String>,     // which face changed it last: cli or web
    #[db(since = 2)] pub at: Option<Millis>,
}
```

- Rust (CLI and cell): the derive (`ce-db-rs`), `use parts_schema::{Part, PartIndexes}` for `by_bin`/`by_qty`.
- Browser: `sdk/ts-web/parts-demo.ts` = `ce-wire-gen --records crates/parts-schema/src/lib.rs --lang ts-web -o sdk/ts-web`.
- Store: `schemas/parts-demo.json` = the same with `--lang json -o schemas`; the first open registers it when the
  grant holds `state.partsdemo.admin`.

## Faces

```
parts add NAME BIN QTY [--supplier S]   parts list [--bin B]   parts low [--under 20]
parts set ID QTY   parts take ID N   parts put ID N   parts rm ID   parts watch [--bin B]
parts-demo serve [--listen 127.0.0.1:8790] [--no-door]   # the page, and the typed api behind this machine's door
```

`parts` and `parts-demo` are one program. The page (`web/`, built by `cd web && npm install && npm run build` into
`web/dist/app.js`, which the binary embeds) shows the parts by bin with live counts, a `byBin` query view per bin, a
feed of live changes with who made them, and +/- and add.

The store: `CE_DB_STORE=127.0.0.1:7811 CE_DB_CAP=<cap>` for a dev store (`ce-db-dev-store --grant
<cap>=state.partsdemo.*` from `repo:ce-db-rs`); then the cell offers `/wire`, a WebSocket that pipes ce-wire to that
store, and the page link is `http://127.0.0.1:8790/#cap=<cap>`. Without those variables `ce_db::open` reaches
state.store by name with the wallet grant `state.store:parts-demo@<node16>`, which the store's owners must issue; the
page then uses wire.ws-door (`#cap=...&g=...&w=wss://...`). The app is never `inventory`.

## Api

`parts-demo-api` (`parts.demo` v1): `HealthOp` (`parts.demo:health`), `Status` (`parts.demo:read`: rows, units, low,
watchers, link, each value with its ce-ref), and the shared `CellSpawn/CellInstances/CellStop/CellDeploy/CellBench`
(900-904). Clients in `sdk/ts`, `sdk/ts-web`, `sdk/swift` (ce-wire-gen).

## Proof (2026-11-14, a dev host, dev store, headless Chromium 156)

`docs/proof-2026-11-14/`: `proof.log` (timings), `parts-watch.log` (the CLI watch), screenshots, `shot.cjs` (the
script). The CLI's `take` showed on the page 64 ms after the CLI call started, its `add` 44 ms after; the page's `+`
was printed by `parts watch` 90 ms after the click; the `byBin("A4")` view saw a CLI `set` 61 ms after. All on one
dev store; a run across two real replicas needs the granted app above.

## Test

`cargo test` (the api over ce-wire with a refusal, the Status refs, a CLI write reaching another watcher within a
second, the page and the dev-only pipe), all against ce-db-dev stores.
