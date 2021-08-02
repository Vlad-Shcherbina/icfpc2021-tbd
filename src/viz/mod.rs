use std::sync::{Arc, Mutex};
use std::net::TcpListener;
use crate::dev_server::{serve_forever, Request, ResponseBuilder, HandlerResult};
use crate::prelude::*;
use crate::checker::{CheckPoseRequest, check_pose};
use crate::shake::{ShakeRequest, shake};
use crate::rotate::{RotateRequest, rotate};
// use crate::poses_live::{Scraper};

struct ServerState {
    client: postgres::Client,
}

crate::entry_point!("viz_server", viz_server);
fn viz_server() {
    let state = ServerState { client: crate::db::connect().unwrap() };
    let state = Arc::new(Mutex::new(state));

    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).unwrap();
    eprintln!("Don't forget to run tsc --watch!");
    eprintln!("listening on http://{} ...", addr);

    serve_forever(listener, || {
        let state = Arc::clone(&state);
        move |req, resp| {
            handler(&*state, req, resp)
        }
    });
}

fn handler(state: &Mutex<ServerState>, req: &Request, resp: ResponseBuilder) -> HandlerResult {
    let client = &mut (*state.lock().unwrap()).client;

    if req.path == "/" {
        return resp.code("302 Found")
            .header("Location", "/src/viz/static/viz.html#42")
            .body("");
    }

    // // old version - submit to orgs' server - replaced by submitting to db
    // if let Some(problem_id) = req.path.strip_prefix("/api/submit/") {
    //     assert_eq!(req.method, "POST");
    //     let problem_id: i32 = problem_id.parse().unwrap();

    //     let pose: Pose = serde_json::from_slice(req.body).unwrap();
    //     let res = crate::poses_live::submit_pose(problem_id, &pose);

    //     let body = match res {
    //         Ok(pose_id) => format!(
    //             r#"submitted as <a href="https://poses.live/solutions/{}">{}</a>"#,
    //             pose_id, pose_id),
    //         Err(e) => e,
    //     };

    //     return resp.code("200 OK")
    //         .body(body);
    // }

    if let Some(problem_id) = req.path.strip_prefix("/api/submit/") {
        assert_eq!(req.method, "POST");
        let problem_id: i32 = problem_id.parse().unwrap();
        let problem = load_problem(problem_id);

        let pose: Pose = serde_json::from_slice(req.body).unwrap();
        let check = check_pose(&problem, &pose);
        if !check.valid {
            return resp.code("200 OK").body("Invalid solution was not submitted");
        }
        return match crate::db::write_valid_solution_to_db(
                client, problem_id, &pose, check.dislikes, "visualizer"
            ) {
            Ok(_) => resp.code("200 OK").body("submitted successfully"),
            Err(a) => resp.code("200 OK").body(format!("{}", a)),
        };
    }

    if req.path == "/api/check_pose" {
        assert_eq!(req.method, "POST");
        let req: CheckPoseRequest = serde_json::from_slice(req.body).unwrap();
        let r = check_pose(&req.problem, &req.pose);
        return resp.code("200 OK")
            .body(serde_json::to_vec(&r).unwrap());
    }

    if req.path == "/api/shake" {
        assert_eq!(req.method, "POST");
        let req: ShakeRequest = serde_json::from_slice(req.body).unwrap();
        let r = shake(&req);
        return resp.code("200 OK")
            .body(serde_json::to_vec(&r).unwrap());
    }

    if req.path == "/api/rotate" {
        assert_eq!(req.method, "POST");
        let req: RotateRequest = serde_json::from_slice(req.body).unwrap();
        let r = rotate(&req);
        return resp.code("200 OK")
            .body(serde_json::to_vec(&r).unwrap());
    }

    // // since the server is (almost) down, switch to db
    // if let Some(problem_id) = req.path.strip_prefix("/api/highscore/") {
    //     assert_eq!(req.method, "GET");
    //     let problem_id: i32 = problem_id.parse().unwrap();

    //     let mut scraper = Scraper::new();
    //     let problem_info = scraper.problem_info(problem_id);

    //     let body = match problem_info.highscore() {
    //         Some(PoseInfo { id, er: EvaluationResult::Valid { dislikes }} ) => format!(
    //             r#"Least dislikes: {}, at <a href="https://poses.live/solutions/{}">{}</a>"#,
    //             dislikes, id, id),
    //         Some(PoseInfo { .. }) => unreachable!(),
    //         None => "No previous valid solutions".to_string(),
    //     };

    //     return resp.code("200 OK")
    //         .body(body);
    // }

    if let Some(problem_id) = req.path.strip_prefix("/api/solution_list/") {
        assert_eq!(req.method, "GET");
        let problem_id: i32 = problem_id.parse().unwrap();
        let v = crate::db::get_solutions_stats_by_problem(client, problem_id).unwrap();
        return resp.code("200 OK").body(serde_json::to_string(&v).unwrap());
    }

    if let Some(solution_id) = req.path.strip_prefix("/api/get_pose/") {
        assert_eq!(req.method, "GET");
        let solution_id: i32 = solution_id.parse().unwrap();
        return match crate::db::get_solution_by_id(client, solution_id).unwrap() {
            None => resp.code("400 Bad Request").body(format!("Wrong solution id {}", solution_id)),
            Some(p) => resp.code("200 OK").body(serde_json::to_vec(&p).unwrap()),
        };
    }

    if let Some(problem_id) = req.path.strip_prefix("/api/tgt_bonuses/") {
        assert_eq!(req.method, "GET");
        let problem_id: i32 = problem_id.parse().unwrap();
        let tgts = crate::db::get_target_bonuses_by_problem(client, problem_id).unwrap();
        dbg!(&tgts);
        return resp.code ("200 OK")
            .body(serde_json::to_vec(&tgts).unwrap());
    }


    // // since the server is (almost) down, switch to db
    // if let Some(pose_id) = req.path.strip_prefix("/api/get_pose/") {
    //     assert_eq!(req.method, "GET");

    //     let mut scraper = Scraper::new();
    //     let pose = scraper.get_pose_by_id(pose_id);

    //     return match pose {
    //         Some(valid_pose) => resp.code("200 OK").body(serde_json::to_vec(&valid_pose).unwrap()),
    //         None => resp.code("404 Not Found").body(vec![]),
    //     }
    // }

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
                "ts" => "text/plain",
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
