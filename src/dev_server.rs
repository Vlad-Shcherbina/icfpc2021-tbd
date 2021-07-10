// Web server that's only intended to serve localhost.
// Prioritizes simplicity over security or compatibility.

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

crate::entry_point!("dev_server_example", dev_server_example);

pub fn dev_server_example() {
    use std::sync::{Arc, Mutex};

    let state = Arc::new(Mutex::new(0));

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    eprintln!("serving at http://127.0.0.1:8000 ...");

    serve_forever(listener, || {
        let state = Arc::clone(&state);
        move |req, resp| {
            match req.path {
                "/" => {
                    let mut cnt = state.lock().unwrap();
                    *cnt += 1;
                    resp.code("200 OK")
                        .header("Content-Type", "text/html; charset=utf8")
                        .body(format!("hello, <b>{}</b>!", *cnt))
                }
                _ => resp.code("404 Not Found").body("not found"),
            }
        }
    })
}

pub fn serve_forever<F, H>(listener: TcpListener, handler_factory: F) -> !
where
    F: Fn() -> H,
    H: FnMut(&Request, ResponseBuilder) -> HandlerResult,
    H: Send + 'static,
{
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            }
        };

        let handler = handler_factory();

        let addr = stream.peer_addr().unwrap();
        let mut name = addr.to_string();
        const MAX_PTHREAD_NAME_LEN: usize = 15;
        if name.len() > MAX_PTHREAD_NAME_LEN {
            name = format!("~{}", &name[name.len() - MAX_PTHREAD_NAME_LEN + 1..]);
        }
        std::thread::Builder::new().name(name).spawn(move || {
            eprintln!("{:?} connected", addr);
            if let Err(e) = serve_request_stream(stream, handler) {
                eprintln!("{} {}", addr, e);
            }
            eprintln!("{} disconnected", addr);
        }).unwrap();
    }
    unreachable!()
}

fn serve_request_stream<H>(mut stream: TcpStream, mut handler: H) -> std::io::Result<()>
where
    H: FnMut(&Request, ResponseBuilder) -> HandlerResult,
{
    let mut buf = Vec::new();
    while let Some(req) = next_request(&mut stream, &mut buf)? {
        let resp = ResponseBuilder { stream };
        match handler(&req, resp)? {
            Some(s) => stream = s,
            None => break,
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    // TODO: headers
    pub body: &'a [u8],
}

fn next_request<'a>(
    stream: &mut TcpStream,
    buf: &'a mut Vec<u8>,
) -> std::io::Result<Option<Request<'a>>> {
    buf.clear();
    let mut len = 0;
    let mut state = 0;
    let header_block_end = 'outer: loop {
        if len == buf.len() {
            buf.resize(if len == 0 { 1024 } else { len * 2 }, 0u8);
        }
        let n = stream.read(&mut buf[len..])?;
        if n == 0 {
            return Ok(None);
        }
        len += n;

        #[allow(clippy::needless_range_loop)]
        for i in len - n .. len {
            match buf[i] {
                b'\n' => {
                    state += 1;
                    if state == 2 {
                        break 'outer i + 1;
                    }
                }
                b'\r' => {}
                _ => state = 0,
            }
        }
    };
    // If there is Content-Length or Transfer-Encoding header, read body
    // (https://tools.ietf.org/html/rfc7230#section-3.3)
    // TODO: Transfer-Encoding is not supported yet
    
    let first_line_end = buf.iter().position(|&b| b == b'\n').unwrap();

    let headers = std::str::from_utf8(&buf[first_line_end + 1 .. header_block_end - 1]).unwrap();
    let q = parse_headers(headers)
        .find(|(name, _value)| name.eq_ignore_ascii_case("Content-Length"));
    let body = if let Some((_, content_length)) = q {
        let content_length: usize = content_length.parse().unwrap();
        assert!(len <= header_block_end + content_length);
        buf.resize(header_block_end + content_length, 0u8);
        stream.read_exact(&mut buf[len..])?;
        &buf[header_block_end..]
    } else {
        assert_eq!(header_block_end, len);
        b""
    };

    let first_line = std::str::from_utf8(&buf[..first_line_end]).unwrap().trim_end();
    let mut parts = first_line.split(' ');
    let method = parts.next().unwrap();
    let path = parts.next().unwrap();
    eprintln!("{} {} {}", stream.peer_addr().unwrap(), method, path);
    let http_version = parts.next().unwrap();
    assert_eq!(http_version, "HTTP/1.1");
    Ok(Some(Request {
        method,
        path,
        body,
    }))
}

fn parse_headers(headers: &str) -> impl Iterator<Item=(&str, &str)> {
    headers.trim_end().split('\n').map(|h| {
        let (name, value) = h.trim_end().split_once(":").unwrap();
        (name, value.trim())
    })
}

pub struct ResponseBuilder {
    stream: TcpStream,
}

impl ResponseBuilder {
    pub fn code(self, s: &str) -> ResponseHeadersBuilder {
        let mut buf = Vec::with_capacity(1024);
        write!(buf, "HTTP/1.1 {}\r\n", s).unwrap();
        ResponseHeadersBuilder {
            stream: self.stream,
            buf,
        }
    }
}

#[must_use]
pub struct ResponseHeadersBuilder {
    stream: TcpStream,
    buf: Vec<u8>,
}

impl ResponseHeadersBuilder {
    fn status_message(&self) -> &str {
        let line_end = self.buf.iter().position(|&b| b == b'\n').unwrap();
        let line = std::str::from_utf8(&self.buf[..line_end]).unwrap();
        line.strip_prefix("HTTP/1.1 ").unwrap().trim_end()
    }

    fn print_status_message(&self) {
        eprintln!("{} {}", self.stream.peer_addr().unwrap(), self.status_message());
    }

    pub fn header(mut self, name: &str, value: impl std::fmt::Display) -> Self {
        write!(self.buf, "{}: {}\r\n", name, value).unwrap();
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> HandlerResult {
        let body = body.as_ref();
        write!(self.buf, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        self.stream.write_all(&self.buf)?;
        self.print_status_message();
        self.stream.write_all(body)?;
        Ok(Some(self.stream))
    }
}

// What we get when we complete the request.
// Some(stream) means that the connection can be reused for further requests
// (HTTP 1.1 persistent connections).
pub type HandlerResult = std::io::Result<Option<TcpStream>>;
