//! Made by cell.forge: this cell serves many callers (ce-fleet standards/cells-serve-many.md). State and limits are
//! per caller, a call without a caller is refused, callers run at once, and the code names no person or home path.
use parts_demo_api::*;
use parts_demo::{Cell, Config, Owner};
use std::sync::Arc;

fn run(c: &Cell, who: &str) -> Result<RunResp, RunError> {
    c.run(who, RunReq { input: "x".into() })
}

#[test]
fn state_is_per_caller() {
    let c = Cell::new(Config::default());
    for _ in 0..3 {
        run(&c, "aa").unwrap();
    }
    run(&c, "bb").unwrap();
    assert_eq!(c.calls(&Owner::of("aa").unwrap()), 3);
    assert_eq!(c.calls(&Owner::of("bb").unwrap()), 1);
    assert_eq!(c.calls(&Owner::of("cc").unwrap()), 0);
}

#[test]
fn no_caller_is_refused() {
    let c = Cell::new(Config::default());
    assert!(matches!(run(&c, ""), Err(RunError::Invalid(_))));
}

#[test]
fn limits_are_per_caller() {
    let c = Cell::new(Config { calls_per_min: 2, ..Config::default() });
    run(&c, "aa").unwrap();
    run(&c, "aa").unwrap();
    assert!(matches!(run(&c, "aa"), Err(RunError::Limited(_))));
    assert!(run(&c, "bb").is_ok(), "one caller's limit slowed another");
}

#[test]
fn memory_is_bounded() {
    let c = Cell::new(Config { max_owners: 2, ..Config::default() });
    run(&c, "aa").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    run(&c, "bb").unwrap();
    run(&c, "cc").unwrap();
    assert_eq!(c.calls(&Owner::of("aa").unwrap()), 0, "the least recently seen caller is dropped");
    assert_eq!(c.calls(&Owner::of("cc").unwrap()), 1);
}

#[test]
fn callers_run_at_once() {
    let c = Arc::new(Cell::new(Config::default()));
    let hs: Vec<_> = (0..8)
        .map(|i| {
            let c = c.clone();
            std::thread::spawn(move || {
                for _ in 0..50 {
                    run(&c, &format!("k{i}")).unwrap();
                }
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    for i in 0..8 {
        assert_eq!(c.calls(&Owner::of(&format!("k{i}")).unwrap()), 50);
    }
}

#[test]
fn code_names_no_person_or_home() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").into());
    for f in ["src/lib.rs", "src/main.rs"] {
        let p = format!("{dir}/{f}");
        let Ok(t) = std::fs::read_to_string(&p) else { continue };
        for bad in ["/home/", "/Users/", "~/", "leif", "asdfghj"] {
            assert!(!t.to_lowercase().contains(bad), "{p} contains {bad:?}: a cell serves anyone (cells-serve-many.md)");
        }
    }
}
