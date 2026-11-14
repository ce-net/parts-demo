//! The page: one HTML file and its script (built from web/ by esbuild, embedded here), and in dev mode a `/wire`
//! WebSocket that pipes ce-wire frames to the dev store unchanged, the way wire.ws-door pipes them to a node's door.
//!
//! In production the page reaches state.store through wire.ws-door instead (`w=` in the link fragment) and this cell
//! offers no `/wire`; the pipe exists only when `CE_DB_STORE` names a dev store. The grant and cap travel in the link
//! fragment, which a browser never sends to a server. Bind loopback or the tailnet address, never 0.0.0.0.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

const INDEX: &str = include_str!("../../../web/index.html");
const APP_JS: &str = include_str!("../../../web/dist/app.js");
const MICHROMA: &[u8] = include_bytes!("../../../web/fonts/michroma.woff2");

/// Serve the page on `listen`; `store` is the dev store `/wire` pipes to (None: no pipe).
pub fn serve(listen: &str, store: Option<String>) -> std::io::Result<()> {
    let l = TcpListener::bind(listen)?;
    eprintln!("parts.demo page on http://{}/ (dev pipe to {})", l.local_addr()?, store.as_deref().unwrap_or("none: use wire.ws-door"));
    for c in l.incoming().flatten() {
        let store = store.clone();
        std::thread::spawn(move || {
            let _ = conn(c, store.as_deref());
        });
    }
    Ok(())
}

/// The request head, peeked (left in the socket for the WebSocket handshake).
fn head(c: &TcpStream) -> std::io::Result<String> {
    let mut buf = [0u8; 8192];
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let n = c.peek(&mut buf)?;
        let s = String::from_utf8_lossy(&buf[..n]).into_owned();
        if s.contains("\r\n\r\n") || n == buf.len() {
            return Ok(s);
        }
        if n == 0 || Instant::now() > until {
            return Err(std::io::Error::other("no request head"));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn reply(mut c: TcpStream, status: &str, ctype: &str, body: &[u8]) -> std::io::Result<()> {
    // drain the request so the close is clean
    let mut sink = [0u8; 8192];
    c.set_read_timeout(Some(Duration::from_millis(50)))?;
    let _ = c.read(&mut sink);
    let h = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    c.write_all(h.as_bytes())?;
    c.write_all(body)?;
    c.flush()
}

fn conn(c: TcpStream, store: Option<&str>) -> std::io::Result<()> {
    let h = head(&c)?;
    let first = h.lines().next().unwrap_or("");
    let mut w = first.split_whitespace();
    let (method, path) = (w.next().unwrap_or(""), w.next().unwrap_or("/"));
    let path = path.split(['?', '#']).next().unwrap_or("/");
    let upgrade = h.lines().any(|l| l.to_ascii_lowercase().starts_with("upgrade:") && l.to_ascii_lowercase().contains("websocket"));
    if method != "GET" {
        return reply(c, "405 Method Not Allowed", "text/plain", b"GET only\n");
    }
    match (path, upgrade, store) {
        ("/wire", true, Some(s)) => pipe(c, s),
        ("/wire", _, None) => reply(c, "404 Not Found", "text/plain", b"no dev pipe here: this page reaches state.store through wire.ws-door (w= in the link)\n"),
        ("/" | "/index.html", false, _) => reply(c, "200 OK", "text/html; charset=utf-8", INDEX.as_bytes()),
        ("/app.js", false, _) => reply(c, "200 OK", "text/javascript; charset=utf-8", APP_JS.as_bytes()),
        ("/fonts/michroma.woff2", false, _) => reply(c, "200 OK", "font/woff2", MICHROMA),
        _ => reply(c, "404 Not Found", "text/plain", b"not found\n"),
    }
}

/// WebSocket <-> TCP, bytes unchanged in both directions; either side closing ends both.
fn pipe(c: TcpStream, store: &str) -> std::io::Result<()> {
    use tungstenite::{Error, Message};
    let up = TcpStream::connect(store)?;
    up.set_nodelay(true)?;
    let mut ws = tungstenite::accept(c).map_err(|e| std::io::Error::other(e.to_string()))?;
    ws.get_mut().set_read_timeout(Some(Duration::from_millis(15)))?;
    ws.get_mut().set_nodelay(true)?;
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let mut up_r = up.try_clone()?;
    std::thread::spawn(move || {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            match up_r.read(&mut buf) {
                Ok(0) | Err(_) => return,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        return;
                    }
                }
            }
        }
    });
    let mut up_w = up;
    'outer: loop {
        match ws.read() {
            Ok(Message::Binary(b)) => {
                if up_w.write_all(&b).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
        loop {
            match rx.try_recv() {
                Ok(b) => {
                    if ws.send(Message::Binary(b)).is_err() {
                        break 'outer;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    let _ = ws.close(None);
                    let _ = ws.flush();
                    break 'outer;
                }
            }
        }
    }
    let _ = up_w.shutdown(std::net::Shutdown::Both);
    Ok(())
}
