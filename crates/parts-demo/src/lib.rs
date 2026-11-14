//! The one function of `parts.demo`: show a parts inventory kept in state.store through ce-db, live in two faces (the
//! `parts` CLI and the page `serve` hands out), so a change made in either appears in the other within a second.
//! The schema is `parts-schema`; the page's types are generated from it (sdk/ts-web/parts-demo.ts).
//!
//! The store: `CE_DB_STORE` + `CE_DB_CAP` name a dev store (`ce-db-dev-store`); without them, `ce_db::open` finds
//! state.store by name with the wallet grant of app `parts-demo` (a grant the store's owners have to issue first). The
//! app is `parts-demo`, never `inventory`, so this demo cannot touch real stock.
use ce_db::prelude::*;
use ce_db::{Change, Collection};
use parts_demo_api::*;
use parts_schema::{Part, PartIndexes, LOW};

pub mod web;

pub const APP: &str = "parts-demo";

/// Open the demo's collection (see the crate doc for where the store comes from).
pub fn parts() -> Result<Collection<Part>, DbError> {
    ce_db::open(APP)?.collection::<Part>()
}

/// The cell's state: the collection, opened once, its mirror shared by every call.
pub struct Cell {
    pub parts: Collection<Part>,
}

impl Cell {
    pub fn new(parts: Collection<Part>) -> Cell {
        Cell { parts }
    }

    pub fn status(&self) -> Result<StatusResp, StatusError> {
        let all = self.parts.by_qty(..).fetch().map_err(|e| match e {
            DbError::Unavailable { .. } => StatusError::Unavailable(e.to_string()),
            e => StatusError::Failed(e.to_string()),
        })?;
        let s = self.parts.stats();
        let src = |label: &str| Ref { label: label.into(), kind: "cell".into(), to: format!("state.store/List#{APP}/parts") };
        Ok(StatusResp {
            app: APP.into(),
            collection: "parts".into(),
            rows: all.len() as u64,
            units: all.iter().map(|p| p.qty as u64).sum(),
            low: all.iter().filter(|p| p.qty < LOW).count() as u64,
            watchers: s.watchers as u32,
            link: format!("{:?}", s.link),
            refs: ["app", "collection", "rows", "units", "low", "watchers", "link"].iter().map(|l| src(l)).collect(),
        })
    }
}

/// One line of `parts list`.
pub fn line(p: &Part) -> String {
    format!("{}  {:<6} {:>6}  {}{}", p.id, p.bin, p.qty, p.name, p.supplier.as_deref().map(|s| format!("  ({s})")).unwrap_or_default())
}

fn clock(ms: i64) -> String {
    let s = (ms / 1000).rem_euclid(86_400);
    format!("{:02}:{:02}:{:02}.{:03}", s / 3600, (s / 60) % 60, s % 60, ms.rem_euclid(1000))
}

/// One line of `parts watch` (UTC clock, the change, the row, which face made it).
pub fn change_line(c: &Change<Part>) -> String {
    let now = Millis::now().0;
    let by = |p: &Part| p.by.as_deref().map(|b| format!("  by {b}")).unwrap_or_default();
    match c {
        Change::Added(p) => format!("{}  added    {:<6} {:<24} qty {}{}", clock(now), p.bin, p.name, p.qty, by(p)),
        Change::Changed { old, new } if old.qty != new.qty => format!("{}  changed  {:<6} {:<24} qty {} -> {}{}", clock(now), new.bin, new.name, old.qty, new.qty, by(new)),
        Change::Changed { new, .. } => format!("{}  changed  {:<6} {:<24} qty {}{}", clock(now), new.bin, new.name, new.qty, by(new)),
        Change::Removed(k) => format!("{}  removed  {k}", clock(now)),
        Change::Synced => format!("{}  synced   (live from here)", clock(now)),
    }
}

/// Stamp a write from the CLI.
pub fn by_cli(p: &mut Part) {
    p.by = Some("cli".into());
    p.at = Some(Millis::now());
}
