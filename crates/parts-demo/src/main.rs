//! `parts-demo serve`: the parts.demo cell behind this machine's door, gated by ce-cap (default deny). Std threads, no async
//! runtime: one thread at rest, about 0.2 MB PSS per process (site-cells DESIGN.md, 2026-10-18).
use parts_demo_api::*;
use ce_wire::blocking::Server;
use std::time::Instant;

fn serve() -> Result<(), String> {
    let me = ce_gate::node_identity().map_err(|e| e.to_string())?;
    let started = Instant::now();
    let gate = ce_gate::Gate::<PartsDemo>::new(me.node_id(), ce_gate::mesh_roots()).into_fn();
    // one Cell for every caller; each call is scoped to the key the gate verified (standards/cells-serve-many.md)
    let cell = parts_demo::Cell::new(parts_demo::Config::from_env());
    Server::<PartsDemo>::new(gate)
        .on::<HealthOp, _>(move |_: (), _| Ok(Health { ok: true, version: env!("CARGO_PKG_VERSION").into(), uptime_s: started.elapsed().as_secs() }))
        .on::<Run, _>(move |r: RunReq, c: &ce_wire::Caller| Ok(cell.run(&c.name, r)))
        .on::<CellSpawn, _>(|r: SpawnReq, _| Ok(ce_wire::cellops::spawn(&r.node, r.n).map(|ids| SpawnResp { ids }).map_err(CellOpsError::Failed)))
        .on::<CellInstances, _>(|r: InstancesReq, _| Ok(ce_wire::cellops::instances(&r.node).map(|ids| InstancesResp { ids }).map_err(CellOpsError::Failed)))
        .on::<CellStop, _>(|r: StopIdReq, _| Ok(ce_wire::cellops::stop(&r.id).map(|stopped| StopIdResp { stopped }).map_err(CellOpsError::Failed)))
        .on::<CellDeploy, _>(|r: DeployReq, _| Ok(ce_wire::cellops::deploy(&r.node, &r.version).map(|version| DeployResp { ok: true, version }).map_err(CellOpsError::Failed)))
        .on::<CellBench, _>(|r: BenchReq, _| Ok(ce_wire::cellops::bench(&r.op, r.n).map(|(p50, p95, errors)| BenchResp { p50_ms: p50, p95_ms: p95, errors }).map_err(CellOpsError::Failed)))
        .serve_door()
        .map_err(|e| e.to_string())
}

fn main() {
    let r = match std::env::args().nth(1).as_deref() {
        Some("serve") => serve(),
        _ => Err("usage: parts-demo serve".into()),
    };
    if let Err(e) = r {
        eprintln!("parts-demo: {e}");
        std::process::exit(1);
    }
}
