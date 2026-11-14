//! The schema of the parts-demo app: the one source. Rust uses this crate; the page's TypeScript and the registered
//! JSON schema are generated from this file:
//! `ce-wire-gen --records crates/parts-schema/src/lib.rs --lang ts-web -o sdk/ts-web` (and `--lang json -o schemas`).
use ce_db_core::prelude::*;

/// A part on a shelf: one row of the parts-demo inventory.
#[derive(Record, Clone, Debug, PartialEq)]
#[db(app = "parts-demo", collection = "parts", version = 2)]
pub struct Part {
    #[db(key)]
    pub id: Id,
    /// What it is.
    #[db(max = 120)]
    pub name: String,
    /// The shelf bin it lives in.
    #[db(index, max = 16)]
    pub bin: String,
    /// How many are in the bin.
    #[db(index, ordered)]
    pub qty: u32,
    /// Who sells it.
    #[db(since = 2, max = 80)]
    pub supplier: Option<String>,
    /// Which face changed it last: `cli` or `web`.
    #[db(since = 2, max = 16)]
    pub by: Option<String>,
    /// When it last changed.
    #[db(since = 2)]
    pub at: Option<Millis>,
}

impl Part {
    pub fn new(name: &str, bin: &str, qty: u32) -> Part {
        Part { id: Id::new(), name: name.into(), bin: bin.into(), qty, supplier: None, by: None, at: None }
    }
}

/// A bin is low below this many.
pub const LOW: u32 = 20;
