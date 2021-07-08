crate::entry_point!("manpages/http", http);

use http::header;
use simple_server::{Method, Server, StatusCode};

#[allow(dead_code)]
pub fn int_div_round_up_i128(dividend: i128, divisor: i128) -> i128 {
    (dividend + (divisor - 1)) / divisor
}

#[allow(dead_code)]
pub fn int_div_round_up_i32(dividend: i32, divisor: i32) -> i32 {
    (dividend + (divisor - 1)) / divisor
}

#[allow(dead_code)]
pub fn int_div_round_up_u128(dividend: u128, divisor: u128) -> u128 {
    (dividend + (divisor - 1)) / divisor
}

#[allow(dead_code)]
pub fn int_div_round_up_u32(dividend: u32, divisor: u32) -> u32 {
    (dividend + (divisor - 1)) / divisor
}

pub fn diagonal_encode(x: &str) -> u32 {
    let alpha = "qazwsxedcrfvtgbyhnujmik,ol.p;/";
    let mut res = 0;
    if x.len() > 4 {
        return 68080;
    }
    for (i, c) in x.chars().rev().enumerate() {
        let charpos = alpha.find(c).unwrap_or(8) as u32;
        let triple_id = int_div_round_up_u32(charpos + 1, 3);
        let mul: u32 = 10_u32.pow(i as u32);
        print!("({},{},{}) ", charpos, triple_id, mul);
        res += mul * (triple_id % 10);
    }
    res
}

pub fn stov(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

fn http() {
    eprintln!("Hello from scratch");

    let host = "localhost";
    let port_string = format!("{}", diagonal_encode("icfp"));
    let port = port_string.as_str();

    let server = Server::new(|req, mut resp| {
        eprintln!("Req {} {}", req.method(), req.uri());
        resp.header(header::CONTENT_TYPE, "application/json".as_bytes());
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/hello") => Ok(resp.body(stov(r#"{"greeting": "hello, world!"}"#))?),
            (_, _) => {
                resp.header(header::CONTENT_TYPE, "text/html; charset=utf8".as_bytes());
                resp.status(StatusCode::NOT_FOUND);
                Ok(resp.body(stov("<h1>🤷 ЧОЧ</h1><p>Unicode (UTF-8, really) is fine.</p>"))?)
            }
        }
    });

    server.listen(host, port);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_our_integer_division_rounds_up() {
        assert_eq!(int_div_round_up_i32(3, 3), 1);
        assert_eq!(int_div_round_up_i32(2, 3), 1);
        assert_eq!(int_div_round_up_i32(1, 3), 1);
        assert_eq!(int_div_round_up_i32(0, 3), 0);
        assert_eq!(int_div_round_up_i32(-1, 3), 0);
    }
}
