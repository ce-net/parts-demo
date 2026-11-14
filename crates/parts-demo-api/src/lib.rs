//! `parts.demo`: the slice 1 demo of the ce-db typed collections (repo:state-store-cell/docs/DB-SDK-DESIGN.md section
//! 9): a parts inventory in state.store (app `parts-demo`, collection `parts`, schema crate `parts-schema`), used from a
//! Rust CLI and from a page this cell serves, which update each other live. Types only: depend on this crate to call
//! the cell; TypeScript and Swift clients are in sdk/ (ce-wire-gen). Op 1 is the health contract every cell exports.

ce_wire::api! {
    api PartsDemo = "parts.demo", version 1;
    types {
        /// Health contract: these three fields, in this order, in every cell.
        pub struct Health { pub ok: bool, pub version: String, pub uptime_s: u64 }
        /// Where a shown value came from (ce-ref): kind msg, goal, commit, file, url, api or cell.
        pub struct Ref { pub label: String, pub kind: String, pub to: String }
        /// The demo's live view of its collection: rows and units in the mirror, open watchers, the link state.
        pub struct StatusResp { pub app: String, pub collection: String, pub rows: u64, pub units: u64, pub low: u64, pub watchers: u32, pub link: String, pub refs: Vec<Ref> }
        pub enum StatusError { Unavailable(String), Failed(String) }
        pub struct SpawnReq { pub node: String, pub n: u32 }
        pub struct SpawnResp { pub ids: Vec<String> }
        pub struct InstancesReq { pub node: String }
        pub struct InstancesResp { pub ids: Vec<String> }
        pub struct StopIdReq { pub id: String }
        pub struct StopIdResp { pub stopped: bool }
        pub struct DeployReq { pub node: String, pub version: String }
        pub struct DeployResp { pub ok: bool, pub version: String }
        pub struct BenchReq { pub op: String, pub n: u32 }
        pub struct BenchResp { pub p50_ms: f64, pub p95_ms: f64, pub errors: u32 }
        pub enum CellOpsError { Invalid(String), Failed(String) }
    }
    ops {
        1 => HealthOp(()) -> Health, needs "parts.demo:health";
        2 => Status(()) -> Result<StatusResp, StatusError>, needs "parts.demo:read";
        900 => CellSpawn(SpawnReq) -> Result<SpawnResp, CellOpsError>, needs "parts.demo:spawn";
        901 => CellInstances(InstancesReq) -> Result<InstancesResp, CellOpsError>, needs "parts.demo:instances";
        902 => CellStop(StopIdReq) -> Result<StopIdResp, CellOpsError>, needs "parts.demo:stop";
        903 => CellDeploy(DeployReq) -> Result<DeployResp, CellOpsError>, needs "parts.demo:deploy";
        904 => CellBench(BenchReq) -> Result<BenchResp, CellOpsError>, needs "parts.demo:bench";
    }
}
