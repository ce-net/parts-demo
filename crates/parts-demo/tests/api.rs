//! Every op through a real ce-wire connection; a caller without the right is refused.
use parts_demo_api::*;
use ce_wire::{Caller, Client, Server, WireError};

async fn serve(rights: &'static [&'static str]) -> std::net::SocketAddr {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let a = l.local_addr().unwrap();
    let cell = std::sync::Arc::new(parts_demo::Cell::new(parts_demo::Config::default()));
    let s = Server::<PartsDemo>::new(move |_| Ok(Caller { name: "t".into(), rights: rights.iter().map(|r| r.to_string()).collect() }))
        .on::<HealthOp, _, _>(|_: (), _| async move { Ok(Health { ok: true, version: "t".into(), uptime_s: 0 }) })
        .on::<Run, _, _>(move |r: RunReq, c: Caller| { let out = cell.run(&c.name, r); async move { Ok(out) } })
        .on::<CellSpawn, _, _>(|_: SpawnReq, _| async move { Ok(Ok(SpawnResp { ids: vec!["1".into()] })) })
        .on::<CellInstances, _, _>(|_: InstancesReq, _| async move { Ok(Ok(InstancesResp { ids: vec!["1".into()] })) })
        .on::<CellStop, _, _>(|_: StopIdReq, _| async move { Ok(Ok(StopIdResp { stopped: true })) })
        .on::<CellDeploy, _, _>(|_: DeployReq, _| async move { Ok(Ok(DeployResp { ok: true, version: "v".into() })) })
        .on::<CellBench, _, _>(|_: BenchReq, _| async move { Ok(Ok(BenchResp { p50_ms: 1.0, p95_ms: 2.0, errors: 0 })) });
    tokio::spawn(s.serve(l));
    a
}

#[tokio::test]
async fn ops_and_refusal() {
    let c = Client::<PartsDemo>::connect(serve(&["parts.demo:health", "parts.demo:run", "parts.demo:spawn", "parts.demo:instances", "parts.demo:stop", "parts.demo:deploy", "parts.demo:bench"]).await, vec![]).await.unwrap();
    assert!(c.call::<HealthOp>(&()).await.unwrap().ok);
    assert_eq!(c.call::<Run>(&RunReq { input: "x".into() }).await.unwrap(), Ok(RunResp { output: "x".into(), refs: vec![Ref { label: "input".into(), kind: "cell".into(), to: "parts.demo/Run".into() }] }));
    assert_eq!(c.call::<Run>(&RunReq { input: String::new() }).await.unwrap(), Err(RunError::Invalid("empty input".into())));
    let c = Client::<PartsDemo>::connect(serve(&["parts.demo:health"]).await, vec![]).await.unwrap();
    assert!(matches!(c.call::<Run>(&RunReq { input: "x".into() }).await, Err(WireError::Refused(_))));
    assert!(matches!(c.call::<CellSpawn>(&SpawnReq { node: String::new(), n: 1 }).await, Err(WireError::Refused(_))));
}

/// sdk/ts was generated from this api: a stale client is refused at connect with a schema error (102 of 261 cells
/// had drifted on 2026-10-15). After changing the api run `ce-wire-gen --lang ts|ts-web|swift`.
#[test]
fn sdk_matches_api() {
    // the run-time dir first: build.offload compiles on another node and runs the test here
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").into());
    let p = format!("{dir}/../../sdk/ts/PartsDemo.ts");
    let ts = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}: run ce-wire-gen --lang ts"));
    let want = format!("SCHEMA = {}n;", <PartsDemo as ce_wire::Api>::SCHEMA);
    assert!(ts.contains(&want), "{p} is stale (want {want}): run ce-wire-gen");
}
