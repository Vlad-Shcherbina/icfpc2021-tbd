crate::entry_point!("manpages/http", http);

use chrono::prelude::*;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::sync::Arc;
use std::sync::Mutex;
use std::net::TcpListener;
use crate::dev_server::serve_forever;

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
        res += mul * (triple_id % 10);
    }
    res
}

pub fn qstom(query_string: &str) -> HashMap<String, String> {
    let kvs = query_string.split('&');
    let mut res = HashMap::new();
    for kv_str in kvs {
        let mut kv = kv_str.split('=');
        let k = urldecode(kv.next().unwrap().to_string());
        let v = urldecode(kv.next().unwrap().to_string());
        res.insert(k, v);
    }
    res
}

// https://docs.rs/urldecode/0.1.1/src/urldecode/lib.rs.html#1-21 don't at me
pub fn urldecode(url: String) -> String {
    let mut decoded = String::from("");
    let mut skip = 0;
    for i in 0..url.len() {
        if skip != 0 {
            skip -= 1;
            continue;
        }
        let c: char = url.chars().nth(i).unwrap();
        if c == '%' {
            let left = url.chars().nth(i + 1).unwrap();
            let right = url.chars().nth(i + 2).unwrap();
            let byte = u8::from_str_radix(&format!("{}{}", left, right), 16).unwrap();
            decoded += &(byte as char).to_string();
            skip = 2;
        } else {
            decoded += &c.to_string();
        }
    }
    decoded
}

#[allow(clippy::or_fun_call)]
pub fn with_q_map_mk_naivedatetime(q_string_map: HashMap<String, String>) -> NaiveDateTime {
    let defaults = Utc::now();
    let default_day = defaults.day() + 1;
    let q_map: HashMap<String, u32> = q_string_map
        .iter()
        .map(|(k, v)| (k.clone(), v.parse::<u32>().unwrap_or(0)))
        .collect();
    let year_u32: u32 = *q_map
        .get("y")
        .unwrap_or(&u32::try_from(defaults.year()).unwrap());
    let year = i32::try_from(year_u32).unwrap_or(defaults.year());
    let month = *q_map.get("m").unwrap_or(&defaults.month());
    let month = if month > 0 { month } else { defaults.month() };
    let day = *q_map.get("d").unwrap_or(&default_day);
    let day = if day > 0 { day } else { default_day };
    let hour = *q_map.get("H").unwrap_or(&defaults.hour());
    let min = *q_map.get("M").unwrap_or(&defaults.minute());
    let sec = 0;
    NaiveDate::from_ymd(year, month, day).and_hms(hour, min, sec)
}

type State = BTreeMap<NaiveDateTime, String>;

pub fn state0() -> State {
    let start = NaiveDate::from_ymd(2021, 7, 9).and_hms(12, 00, 00);
    let lightning_end = NaiveDate::from_ymd(2021, 7, 10).and_hms(12, 00, 00);
    let contest_end = NaiveDate::from_ymd(2021, 7, 12).and_hms(12, 00, 00);
    BTreeMap::from_iter([
        (start, "ICFPC 2021 Starts".to_string()),
        (lightning_end, "End of Lightning Round".to_string()),
        (contest_end, "End of Contest".to_string()),
    ])
}

fn http() {
    eprintln!("Hello from scratch");

    let host = "127.0.0.1";
    let port = diagonal_encode("icfp");
    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    eprintln!("Serving at http://{}:{} ...", host, port);

    let state = Arc::new(Mutex::new(state0()));
    serve_forever(listener, || {
        let state = Arc::clone(&state);
        move |req, resp| {
            match (req.method, req.path) {
                ("POST", "/rotate") => {
                    let data = String::from_utf8_lossy(req.body).into_owned();
                    let body = format!("The data you posted was '{}'", data);
                    resp.code("200 OK")
                        .header("Content-Type", "text/html; charset=utf8")
                        .body(body)
                }
                ("GET", "/dashboard/timer") => {
                    let state_contents = &*state.lock().unwrap();
                    if let Ok(serialized) = serde_json::to_string(state_contents) {
                        resp.code("200 OK")
                            .header("Content-Type", "application/json")
                            .body(serialized)
                    } else {
                        resp.code("500 Internal Server Error")
                            .header("Content-Type", "text/html; charset=utf8")
                            .body("<h1>🤷 502</h1><p>D̾ͮ̎ͪ͏̰͖̳̣ą͙̖̘̩̱͙͂t̻̩͙̹̳̳̼̂ͭͧ̈́͡aͫ͏̯͇̠̲͔ ̧̗̳̜̥͌ͩ͗i̙̮͗͑́s̐̑̀̈́͏̬̙̪͍̭̱̙ ̰͇̦̗͂ͮ̍͡ͅc̛̫̪͙̣͔̙͉̣̓ǒ̄҉̥̙̟͕̝r̛̙̫̹͕͒r͕̻̫̼̘̻̂͌ͤ̎͜u̢̞͙͕͐̚ṕ̷̩͕̗̗̞̠̒̌̋t̲̜̺̭͖̝̅ͥ͘ͅe̵̮͖͈̝͔̥͎̎͆ͦ̒ͅd̷͉̙̰͖͉͙̐ͫ</p>")
                    }
                }
                ("GET", "/dashboard/timer/insert") => {
                    let uri: http::Uri = req.path.parse().unwrap();
                    if let Some(qs) = uri.query() {
                        let q_map = qstom(qs);
                        let target = with_q_map_mk_naivedatetime(q_map.clone());
                        let label = q_map.get("label").unwrap_or(&"a timer".to_string()).clone();
                        {
                            let mut state_contents = state.lock().unwrap();
                            state_contents.insert(target, label);
                            eprintln!("State1 {:#?}", *state_contents);
                        }
                        resp.code("200 OK")
                            .header("Content-Type", "text/html; charset=utf8")
                            .body(format!(r##"Inserting {:#?}"##, target))
                    } else {
                        resp.code("422 Unprocessable Entity")
                            .header("Content-Type", "text/html; charset=utf8")
                            .body("<h1>🤷 Ч22</h1><p>Query string is missing.</p>")
                    }
                }
                ("GET", "/dashboard/timer/remove") => {
                    let uri: http::Uri = req.path.parse().unwrap();
                    if let Some(qs) = uri.query() {
                        let q_map = qstom(qs);
                        let target = with_q_map_mk_naivedatetime(q_map);
                        {
                            let state_ref = state.clone();
                            let mut state_contents = state_ref.lock().unwrap();
                            state_contents.remove(&target);
                            eprintln!("State1 {:#?}", *state_contents);
                        }
                        resp.code("200 OK")
                            .header("Content-Type", "text/html; charset=utf8")
                            .body(format!(r##"Removing {:#?}"##, target))
                    } else {
                        resp.code("422 Unprocessable Entity")
                            .header("Content-Type", "text/html; charset=utf8")
                            .body("<h1>🤷 Ч22</h1><p>Query string is missing.</p>")
                    }
                }
                (_, _) => {
                    let pth = crate::util::project_path(format!("public{}", req.path));
                    match std::fs::read(&pth) {
                        Ok(a) if req.method == "GET" => {
                            let typ = match pth.extension().unwrap().to_str().unwrap() {
                                "html" => "text/html; charset=utf8",
                                "css" => "text/css",
                                "ico" => "image/x-icon",
                                _ => panic!("{:?}", pth),
                            };
                            resp.code("200 OK")
                                .header("Content-Type", typ)
                                .body(a)
                        }
                        _ => {
                            resp.code("404 Not Found")
                                .header("Content-Type", "text/html; charset=utf8")
                                .body("<h1>🤷 ЧОЧ</h1><p>Unicode (UTF-8, really) is fine.</p>
                                      <a href='/dashboard.html'>countdown</a>")
                        }
                    }
                }
            }
        }
    });
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

    #[test]
    fn urldecode_isnt_completely_broken() {
        assert_eq!(
            "https://github.com/ /",
            urldecode(String::from("https%3A%2F%2Fgithub.com/%20/"))
        );
    }
}
