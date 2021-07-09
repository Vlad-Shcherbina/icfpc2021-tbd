#![allow(clippy::needless_range_loop)]

use super::dev_server::{serve_forever, Request, ResponseBuilder, HandlerResult};
use crate::util::project_path;

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

type GameList = HashMap<u32, (Vec<Vec<u8>>, Vec<Vec<u8>>)>;

crate::entry_point!("life", life_server);
pub fn life_server() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    eprintln!("Don't forget to run tsc --watch!");
    eprintln!("listening http://127.0.0.1:8000 ...");
    eprintln!("http://127.0.0.1:8000/src/scratches/julie/life/static/life.html");
    let games: GameList = HashMap::new();
    let g = Arc::new(Mutex::new(games));
    serve_forever(listener, || { 
        let m = Arc::clone(&g);
        move |r, p| { life_handler(&m, r, p) } 
    });
}


fn life_handler(games: &Mutex<GameList>, request: &Request, response: ResponseBuilder) -> HandlerResult {
    let mut games = games.lock().unwrap();
    if request.path == "/api/change/" {
        let r: LifeChangeRequest = serde_json::from_slice(request.body).unwrap();
        return life_change_handler(&mut *games, r, response);
    }
    if request.path == "/api/step/" {
        let r: LifeStateRequest = serde_json::from_slice(request.body).unwrap();
        return life_step_handler(&mut *games, r, response);
    }
    if request.path == "/api/get/" {
        let r: LifeStateRequest = serde_json::from_slice(request.body).unwrap();
        return life_get_handler(&mut *games, r, response);
    }
    static_handler(request, response)
}


fn static_handler(request: &Request, response: ResponseBuilder) -> HandlerResult {
    assert_eq!(request.method, "GET");

    let pth = project_path(request.path.strip_prefix('/').unwrap());
    dbg!(&pth);
    match std::fs::read(&pth) {
        Ok(a) => {
            let typ = match pth.extension().unwrap().to_str().unwrap() {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "map" => "text/plain",
                _ => panic!("{:?}", pth),
            };
            response.code("200 OK")
                    .header("Content-Type", typ)
                    .body(a)
        },
        Err(_) => {
            let a = std::fs::read(project_path("src/scratches/julie/life/static/404.html")).unwrap();
            response.code("404 Not Found")
                    .header("Content-Type", "text/html")
                    .body(a)
        },
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LifeStateRequest {
    session: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LifeChangeRequest {
    session: u32,
    x: usize,
    y: usize,
}

#[derive(serde::Serialize, Clone)]
struct Response<'a> {
    field: &'a Vec<Vec<u8>>,
    session: u32,
}


fn life_get_handler(games: &mut GameList, LifeStateRequest { session }: LifeStateRequest, 
        response: ResponseBuilder) -> HandlerResult {
    if session == 0 || !games.contains_key(&session) {
        let s = loop {
            let s: u32 = rand::random();
            if s == 0 || games.contains_key(&s) { continue; }
            break s;
        };
        games.insert(s, (random_field(50, 50), vec![vec![0; 50]; 50]));
        return life_state_handler(&Response { field: &games[&s].0, session: s}, response);
    }
    life_state_handler(&Response { field: &games[&session].0, session }, response)
}


fn life_step_handler(games: &mut GameList, LifeStateRequest { session }: LifeStateRequest, 
        response: ResponseBuilder) -> HandlerResult {
    life_step(&mut games.get_mut(&session).unwrap());
    life_state_handler(&Response { field: &games[&session].0, session }, response)
}


fn life_change_handler(games: &mut GameList, request: LifeChangeRequest, 
        response: ResponseBuilder) -> HandlerResult {
    let field = &mut games.get_mut(&request.session).unwrap().0;
    field[request.y][request.x] = 1 - field[request.y][request.x];
    life_state_handler(&Response { field, session: request.session }, response)
}


fn life_state_handler(r: &Response, response: ResponseBuilder) -> HandlerResult {
    response.code("200 Ok")
            .header("Content-type", "application/json")
            .body(serde_json::to_string(r).unwrap())
}


// ------------------ LIFE ------------------------

fn random_field(width: usize, height: usize) -> Vec<Vec<u8>> {
    let mut v: Vec<Vec<u8>> = vec![vec![0; width]; height];
    for i in 0..height {
        for j in 0..width {
            v[i][j] = rand::random::<u8>() % 2;
        }
    }
    v
}

fn count_alive(field: &[Vec<u8>], i: usize, j: usize) -> i32 {
    let mut count = 0;
    let top = if i > 0 { i - 1 } else { i };
    let bottom = if i < field.len() - 1 { i + 1 } else { i };
    let left = if j > 0 { j - 1 } else { j };
    let right = if j < field[0].len() - 1 { j + 1 } else { j };
    for ii in top..=bottom {
        for jj in left..=right {
            if i == ii && j == jj { continue; }
            count += field[ii][jj] as i32;
        }
    }
    count
}

fn life_step(game: &mut (Vec<Vec<u8>>, Vec<Vec<u8>>)) {
    let first = &mut game.0;
    let second = &mut game.1;
    for i in 0..first.len() {
        for j in 0..first[0].len() {
            second[i][j] = match count_alive(first, i, j) {
                3 => 1,
                2 => first[i][j],
                _ => 0,
            };
        }
    }
    std::mem::swap(first, second);
}
