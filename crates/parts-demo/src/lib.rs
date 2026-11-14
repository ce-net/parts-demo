//! The one function of `parts.demo`: Parts inventory demo of the ce-db typed collections: a Rust CLI and a tailnet page on state.store that update each other live. Keep it one function; a second one is a second cell.
//!
//! Serves many callers (ce-fleet standards/cells-serve-many.md). Every call is scoped to the caller's key, which
//! ce-gate verified and put in `Caller.name`. State and limits are keyed by that owner. There is no default owner,
//! no home path and no person's name in this code, and settings come from the deployment's environment.
use parts_demo_api::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The caller a request belongs to: the key ce-gate verified (hex of the requester's public key).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Owner(String);

impl Owner {
    /// From `Caller.name`. An empty or oversized name is refused: no call runs without a caller.
    pub fn of(caller: &str) -> Result<Owner, RunError> {
        if caller.is_empty() || caller.len() > 256 {
            return Err(RunError::Invalid("no caller identity".into()));
        }
        Ok(Owner(caller.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Per-deployment settings, read from the environment at start (`PARTS_DEMO_*`). The defaults serve many users.
#[derive(Clone, Debug)]
pub struct Config {
    /// Calls one caller may make per minute; one busy caller never slows another.
    pub calls_per_min: u32,
    /// Distinct callers kept in memory at once; the least recently seen is dropped first.
    pub max_owners: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { calls_per_min: 600, max_owners: 10_000 }
    }
}

impl Config {
    pub fn from_env() -> Config {
        let d = Config::default();
        let n = |k: &str, v: u64| std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(v);
        Config {
            calls_per_min: n("PARTS_DEMO_CALLS_PER_MIN", d.calls_per_min as u64) as u32,
            max_owners: n("PARTS_DEMO_MAX_OWNERS", d.max_owners as u64).max(1) as usize,
        }
    }
}

struct Slot {
    window: Instant,
    calls: u32,
    seen: Instant,
}

/// The cell's state, shared by every connection. Each entry belongs to one owner and no call reads another's.
/// Durable per-owner records go to state.store under a key that starts with the owner (standards/state-lives-in-the-state-db.md).
pub struct Cell {
    cfg: Config,
    owners: Mutex<HashMap<Owner, Slot>>,
}

impl Cell {
    pub fn new(cfg: Config) -> Cell {
        Cell { cfg, owners: Mutex::new(HashMap::new()) }
    }

    /// The one function, for one caller.
    pub fn run(&self, caller: &str, r: RunReq) -> Result<RunResp, RunError> {
        let owner = Owner::of(caller)?;
        self.admit(&owner)?;
        if r.input.is_empty() {
            return Err(RunError::Invalid("empty input".into()));
        }
        // every shown value names its source (ce-ref, tests/refs.rs): this output is the caller's own input
        Ok(RunResp { output: r.input, refs: vec![Ref { label: "input".into(), kind: "cell".into(), to: "parts.demo/Run".into() }] })
    }

    /// Calls this owner made in the current minute.
    pub fn calls(&self, owner: &Owner) -> u32 {
        self.owners.lock().unwrap_or_else(|e| e.into_inner()).get(owner).map(|s| s.calls).unwrap_or(0)
    }

    fn admit(&self, owner: &Owner) -> Result<(), RunError> {
        let now = Instant::now();
        let mut m = self.owners.lock().unwrap_or_else(|e| e.into_inner());
        if !m.contains_key(owner) && m.len() >= self.cfg.max_owners {
            if let Some(old) = m.iter().min_by_key(|(_, s)| s.seen).map(|(o, _)| o.clone()) {
                m.remove(&old);
            }
        }
        let s = m.entry(owner.clone()).or_insert(Slot { window: now, calls: 0, seen: now });
        if now.duration_since(s.window) >= Duration::from_secs(60) {
            s.window = now;
            s.calls = 0;
        }
        s.seen = now;
        if s.calls >= self.cfg.calls_per_min {
            return Err(RunError::Limited(format!("{} calls a minute per caller", self.cfg.calls_per_min)));
        }
        s.calls += 1;
        Ok(())
    }
}
