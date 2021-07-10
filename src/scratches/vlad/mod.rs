use std::net::TcpListener;
use crate::dev_server::{serve_forever, Request, ResponseBuilder, HandlerResult};
use crate::util::project_path;

crate::entry_point!("sse_demo", sse_demo);
fn sse_demo() {
    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).unwrap();
    eprintln!("Don't forget to run tsc --watch!");
    eprintln!("listening on http://{} ...", addr);

    serve_forever(listener, || {
        move |req, resp| {
            if req.path == "/" {
                return resp.code("302 Found")
                    .header("Location", "/src/scratches/vlad/sse.html")
                    .body("");
            }

            if req.path == "/sse" {
                // return resp.code("200 OK").body("");
                let mut sse = resp.code("200 OK").sse()?;
                for _ in 0..5 {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    eprintln!("yo");
                    sse.message()?;
                }
                return Ok(None);
            }

            static_handler(req, resp)
        }
    });
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
                "problem" => "application/json",
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
