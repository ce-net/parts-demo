//! The two faces on one store: a CLI write reaches a watcher live, and the page and its dev pipe are served.
use ce_db::Change;
use parts_schema::{Part, PartIndexes};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

#[test]
fn a_cli_write_reaches_another_watcher_within_a_second() {
    let s = ce_db_dev::serve("127.0.0.1:0", ce_db_dev::Grants::all_of("rw", "parts-demo")).unwrap();
    let open = || ce_db::open_at("parts-demo", &s.addr.to_string(), "rw").unwrap().collection::<Part>().unwrap();
    let (page, cli) = (open(), open());
    let mut w = page.by_bin("A4").watch().unwrap();
    assert_eq!(w.next_timeout(Duration::from_secs(2)), Some(Ok(Change::Synced)));
    let mut p = Part::new("M3x8 screw", "A4", 500);
    parts_demo::by_cli(&mut p);
    let t = Instant::now();
    cli.insert(&p).unwrap();
    let ch = w.next_timeout(Duration::from_secs(2)).expect("an event").unwrap();
    assert!(t.elapsed() < Duration::from_secs(1), "{:?}", t.elapsed());
    assert_eq!(ch, Change::Added(p.clone()));
    assert!(parts_demo::change_line(&ch).contains("added    A4     M3x8 screw               qty 500  by cli"), "{}", parts_demo::change_line(&ch));
    cli.update(p.id.as_str(), |x| x.qty = 499).unwrap();
    let ch = w.next_timeout(Duration::from_secs(2)).expect("an event").unwrap();
    assert!(parts_demo::change_line(&ch).contains("qty 500 -> 499"));
}

fn get(addr: std::net::SocketAddr, path: &str) -> String {
    let mut c = std::net::TcpStream::connect(addr).unwrap();
    write!(c, "GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
    let mut s = String::new();
    c.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn the_page_is_served_and_wire_is_dev_only() {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let a = l.local_addr().unwrap();
    drop(l);
    std::thread::spawn(move || parts_demo::web::serve(&a.to_string(), None));
    std::thread::sleep(Duration::from_millis(200));
    let page = get(a, "/");
    assert!(page.starts_with("HTTP/1.1 200") && page.contains("<title>Parts inventory</title>"));
    assert!(get(a, "/app.js").contains("parts-demo"));
    assert!(get(a, "/wire").starts_with("HTTP/1.1 404"), "no pipe without a dev store");
}
