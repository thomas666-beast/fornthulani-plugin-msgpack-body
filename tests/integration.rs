use fornthulani::{Params, Request, Response, Router, Server, Status, ThulaniError};
use fornthulani_plugin_msgpack_body::MsgPackBody;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::thread;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Item {
    sku: String,
    qty: u32,
}

fn handler(req: &Request, _: &Params) -> Result<Response, ThulaniError> {
    let item: Item = req.msgpack()?;
    Ok(Response::json(Status::OK, &item))
}

fn spawn_echo() -> (String, fornthulani::ServerHandle, thread::JoinHandle<()>) {
    let mut router = Router::new();
    router.post("/echo", handler);

    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let (handle, join) = Server::new(router).spawn(&addr.to_string()).unwrap();
    thread::sleep(std::time::Duration::from_millis(50));
    (format!("http://{addr}"), handle, join)
}

#[test]
fn accepts_valid_msgpack() {
    let (base, handle, join) = spawn_echo();

    let item = Item {
        sku: "ABC".into(),
        qty: 3,
    };
    let bytes = rmp_serde::to_vec_named(&item).unwrap();

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("{base}/echo"))
        .header("content-type", "application/msgpack")
        .body(bytes)
        .send()
        .unwrap();
    assert_eq!(res.status(), 200);

    let echoed: Item = res.json().unwrap();
    assert_eq!(echoed, item);

    handle.shutdown();
    join.join().unwrap();
}

#[test]
fn rejects_wrong_content_type() {
    let (base, handle, join) = spawn_echo();

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("{base}/echo"))
        .header("content-type", "text/plain")
        .body(vec![0x91, 0x01])
        .send()
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(res.text().unwrap().contains("application/msgpack"));

    handle.shutdown();
    join.join().unwrap();
}

#[test]
fn rejects_invalid_msgpack() {
    let (base, handle, join) = spawn_echo();

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!("{base}/echo"))
        .header("content-type", "application/msgpack")
        .body(vec![0xff, 0xff, 0xff, 0xff])
        .send()
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(res.text().unwrap().contains("invalid msgpack"));

    handle.shutdown();
    join.join().unwrap();
}
