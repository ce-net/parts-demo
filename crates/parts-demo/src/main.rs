//! `parts-demo serve [--listen ADDR] [--no-door]`: the parts.demo cell (the page, and its typed api behind this
//! machine's door unless --no-door). The same binary is the CLI (`parts` is the same program):
//!
//!   parts add NAME BIN QTY [--supplier S]     parts list [--bin B]      parts low [--under N]
//!   parts set ID QTY    parts take ID N    parts put ID N    parts rm ID    parts watch [--bin B]
//!
//! Store: `CE_DB_STORE` + `CE_DB_CAP` (a dev store), else state.store by name (see the lib doc).
use ce_db::prelude::*;
use parts_demo_api::*;
use parts_schema::{Part, PartIndexes, LOW};
use std::time::Instant;

fn flag(a: &[String], name: &str) -> Option<String> {
    a.iter().position(|x| x == name).and_then(|i| a.get(i + 1)).cloned()
}

fn pos(a: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for x in a.iter().skip(2) {
        if skip {
            skip = false;
        } else if x.starts_with("--") {
            skip = !matches!(x.as_str(), "--no-door");
        } else {
            out.push(x.clone());
        }
    }
    out
}

fn num(s: Option<&String>, what: &str) -> Result<u32, String> {
    s.ok_or_else(|| format!("{what} missing"))?.parse().map_err(|_| format!("{what}: a whole number"))
}

fn serve(a: &[String]) -> Result<(), String> {
    let listen = flag(a, "--listen").unwrap_or_else(|| "127.0.0.1:8790".into());
    let store = std::env::var("CE_DB_STORE").ok().filter(|s| !s.trim().is_empty());
    if !a.iter().any(|x| x == "--no-door") {
        let started = Instant::now();
        std::thread::spawn(move || {
            let r = (|| -> Result<(), String> {
                let me = ce_gate::node_identity().map_err(|e| e.to_string())?;
                let gate = ce_gate::Gate::<PartsDemo>::new(me.node_id(), ce_gate::mesh_roots()).into_fn();
                let cell = std::sync::Arc::new(parts_demo::Cell::new(parts_demo::parts().map_err(|e| e.to_string())?));
                let bench_cell = cell.clone();
                ce_wire::blocking::Server::<PartsDemo>::new(gate)
                    .on::<HealthOp, _>(move |_: (), _| Ok(Health { ok: true, version: env!("CARGO_PKG_VERSION").into(), uptime_s: started.elapsed().as_secs() }))
                    .on::<Status, _>(move |_: (), _| Ok(cell.status()))
                    .on::<CellSpawn, _>(|r: SpawnReq, _| Ok(ce_wire::cellops::spawn(&r.node, r.n).map(|ids| SpawnResp { ids }).map_err(CellOpsError::Failed)))
                    .on::<CellInstances, _>(|r: InstancesReq, _| Ok(ce_wire::cellops::instances(&r.node).map(|ids| InstancesResp { ids }).map_err(CellOpsError::Failed)))
                    .on::<CellStop, _>(|r: StopIdReq, _| Ok(ce_wire::cellops::stop(&r.id).map(|stopped| StopIdResp { stopped }).map_err(CellOpsError::Failed)))
                    .on::<CellDeploy, _>(|r: DeployReq, _| Ok(ce_wire::cellops::deploy(&r.node, &r.version).map(|version| DeployResp { ok: true, version }).map_err(CellOpsError::Failed)))
                    .on::<CellBench, _>(move |r: BenchReq, _| Ok(ce_wire::cellops::bench(&r.op, r.n, || if r.op == "Status" { bench_cell.status().map(|_| ()).map_err(|e| format!("{e:?}")) } else { Ok(()) }).map(|(p50, p95, errors)| BenchResp { p50_ms: p50, p95_ms: p95, errors }).map_err(CellOpsError::Failed)))
                    .serve_door()
                    .map_err(|e| e.to_string())
            })();
            if let Err(e) = r {
                eprintln!("parts-demo: the typed api is not served: {e}");
            }
        });
    }
    parts_demo::web::serve(&listen, store).map_err(|e| e.to_string())
}

fn run(a: &[String]) -> Result<(), String> {
    let cmd = a.get(1).map(String::as_str).unwrap_or("");
    if cmd == "serve" {
        return serve(a);
    }
    let p = pos(a);
    let parts = || parts_demo::parts().map_err(|e| e.to_string());
    let e = |e: DbError| e.to_string();
    match cmd {
        "add" => {
            let mut x = Part::new(p.first().ok_or("add NAME BIN QTY")?, p.get(1).ok_or("add NAME BIN QTY")?, num(p.get(2), "QTY")?);
            x.supplier = flag(a, "--supplier");
            parts_demo::by_cli(&mut x);
            let id = parts()?.insert(&x).map_err(e)?;
            println!("{id}");
        }
        "list" => {
            let c = parts()?;
            let rows = match flag(a, "--bin") {
                Some(b) => c.by_bin(b).fetch().map_err(e)?,
                None => c.by_qty(..).desc().fetch().map_err(e)?,
            };
            for r in &rows {
                println!("{}", parts_demo::line(r));
            }
            eprintln!("{} parts", rows.len());
        }
        "low" => {
            let under = flag(a, "--under").map(|u| u.parse::<u32>().map_err(|_| "--under N")).transpose()?.unwrap_or(LOW);
            for r in parts()?.by_qty(..under).fetch().map_err(e)? {
                println!("{}", parts_demo::line(&r));
            }
        }
        "set" | "take" | "put" => {
            let id = p.first().ok_or_else(|| format!("{cmd} ID N"))?.clone();
            let n = num(p.get(1), "N")?;
            let after = parts()?
                .update(&id, |x| {
                    x.qty = match cmd {
                        "set" => n,
                        "take" => x.qty.saturating_sub(n),
                        _ => x.qty.saturating_add(n),
                    };
                    parts_demo::by_cli(x);
                })
                .map_err(e)?;
            println!("{}", parts_demo::line(&after));
        }
        "rm" => parts()?.remove(p.first().ok_or("rm ID")?).map_err(e)?,
        "watch" => {
            let c = parts()?;
            let w = match flag(a, "--bin") {
                Some(b) => c.by_bin(b).watch(),
                None => c.watch(),
            }
            .map_err(e)?;
            for ch in w {
                println!("{}", parts_demo::change_line(&ch.map_err(e)?));
            }
        }
        _ => return Err("usage: parts add NAME BIN QTY [--supplier S] | list [--bin B] | low [--under N] | set ID QTY | take ID N | put ID N | rm ID | watch [--bin B] | serve [--listen ADDR] [--no-door]".into()),
    }
    Ok(())
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if let Err(e) = run(&a) {
        eprintln!("parts: {e}");
        std::process::exit(1);
    }
}
