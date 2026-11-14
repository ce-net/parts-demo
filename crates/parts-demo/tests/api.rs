//! Every op through a real ce-wire connection, the cell on a dev store; a caller without the right is refused.
use ce_wire::blocking::{Client, Server};
use ce_wire::{Caller, WireError};
use parts_demo_api::*;
use parts_schema::Part;

fn store() -> ce_db_dev::DevStore {
    ce_db_dev::serve("127.0.0.1:0", ce_db_dev::Grants::all_of("rw", "parts-demo")).unwrap()
}

fn serve(s: &ce_db_dev::DevStore, rights: &'static [&'static str]) -> std::net::SocketAddr {
    let parts = ce_db::open_at("parts-demo", &s.addr.to_string(), "rw").unwrap().collection::<Part>().unwrap();
    let cell = parts_demo::Cell::new(parts);
    let srv = Server::<PartsDemo>::new(move |_| Ok(Caller { name: "t".into(), rights: rights.iter().map(|r| r.to_string()).collect() }))
        .on::<HealthOp, _>(|_: (), _| Ok(Health { ok: true, version: "t".into(), uptime_s: 0 }))
        .on::<Status, _>(move |_: (), _| Ok(cell.status()));
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let a = l.local_addr().unwrap();
    std::thread::spawn(move || srv.serve(l));
    a
}

#[test]
fn health_status_and_refusal() {
    let s = store();
    let c = ce_db::open_at("parts-demo", &s.addr.to_string(), "rw").unwrap().collection::<Part>().unwrap();
    c.insert(&Part::new("M3x8 screw", "A4", 500)).unwrap();
    c.insert(&Part::new("M4 nut", "A4", 12)).unwrap();
    let cl = Client::<PartsDemo>::connect(serve(&s, &["parts.demo:health", "parts.demo:read"]), vec![]).unwrap();
    assert!(cl.call::<HealthOp>(&()).unwrap().ok);
    let st = cl.call::<Status>(&()).unwrap().unwrap();
    assert_eq!((st.rows, st.units, st.low), (2, 512, 1));
    let cl = Client::<PartsDemo>::connect(serve(&s, &["parts.demo:health"]), vec![]).unwrap();
    assert!(matches!(cl.call::<Status>(&()), Err(WireError::Refused(_))));
}

/// sdk/ts was generated from this api: a stale client is refused at connect with a schema error. After changing the
/// api run `ce-wire-gen --lang ts|ts-web|swift`.
#[test]
fn sdk_matches_api() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").into());
    let p = format!("{dir}/../../sdk/ts/PartsDemo.ts");
    let ts = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}: run ce-wire-gen --lang ts"));
    let want = format!("SCHEMA = {}n;", <PartsDemo as ce_wire::Api>::SCHEMA);
    assert!(ts.contains(&want), "{p} is stale (want {want}): run ce-wire-gen");
}
