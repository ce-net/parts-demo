//! Made by cell.forge: every value this cell shows carries its source (ce-ref, ce-load README). A reply field with no
//! ref above it fails here, naming its path; give its record a `refs: Vec<Ref>`.
use parts_demo_api::*;
use parts_demo::{Cell, Config};

#[test]
fn every_shown_value_has_a_ref() {
    let c = Cell::new(Config::default());
    ce_ref::assert_sourced(&c.run("aa", RunReq { input: "x".into() }).unwrap());
}
