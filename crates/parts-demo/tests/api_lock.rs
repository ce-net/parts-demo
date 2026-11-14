//! Made by cell.forge, rewritten on every run. api.lock holds the per-op hashes callers of this version rely on: an
//! api change that would refuse them fails here (fix: `ce-wire-gen freeze`, then `ce-wire-gen adapt`; rust-stack
//! docs/SCHEMA-ADAPTERS.md). The first run writes the lock; commit it.
#[test]
fn api_lock() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").into());
    let lock = std::path::Path::new(&dir).join("../parts-demo-api/api.lock");
    match ce_wire::check_lock::<parts_demo_api::PartsDemo>(&lock) {
        Ok(m) => eprintln!("{m}"),
        Err(e) => panic!("{e}"),
    }
}
