use std::sync::{Arc, Mutex};
use std::net::TcpListener;
use crate::dev_server::{serve_forever, Request, ResponseBuilder, HandlerResult};
use crate::util::*;

struct ServerState {
}

crate::entry_point!("viz_server", viz_server);
fn viz_server() {
    let state = ServerState {};
    let state = Arc::new(Mutex::new(state));

    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).unwrap();
    eprintln!("Don't forget to run tsc --watch!");
    eprintln!("listening http://{} ...", addr);

    serve_forever(listener, || {
        let state = Arc::clone(&state);
        move |req, resp| {
            handler(&*state, req, resp)
        }
    });
}

fn handler(_state: &Mutex<ServerState>, req: &Request, resp: ResponseBuilder) -> HandlerResult {
    if req.path == "/" {
        return resp.code("302 Found")
            .header("Location", "/src/viz/static/viz.html#42")
            .body("");
    }

    static_handler(req, resp)
}

fn static_handler(req: &Request, resp: ResponseBuilder) -> HandlerResult {
    assert_eq!(req.method, "GET");

    let pth = project_path(req.path.strip_prefix('/').unwrap());
    match std::fs::read(&pth) {
        Ok(a) => {
            let typ = match pth.extension().unwrap().to_str().unwrap() {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "map" => "text/plain",
                _ => {
                    eprintln!("warning: unknown file extension {}", pth.to_string_lossy());
                    eprintln!("serving as text/plain");
                    "text/plain"
                }
            };
            resp.code("200 OK")
                .header("Content-Type", typ)
                .body(a)
        },
        Err(_) => {
            resp.code("404 Not Found")
                .header("Content-Type", "text/html")
                .body("not found")
        },
    }
}
