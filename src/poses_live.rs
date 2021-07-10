// A client library for their portal https://poses.live

use crate::prelude::*;

const API_KEY: &str = "81acc597-be90-418c-90aa-0dfac878aeb0";

crate::entry_point!("poses_live_demo", poses_live_demo, _EP1);
fn poses_live_demo() {
    let resp = ureq::get("https://poses.live/api/hello")
        .set("Authorization", &format!("Bearer {}", API_KEY))
        .call().unwrap();
    eprintln!("{:?}", resp);
    assert_eq!(resp.status(), 200);
    let body = resp.into_string().unwrap();
    eprintln!("{}", body);
}

crate::entry_point!("submit_example", submit_example, _EP2);
fn submit_example() {
    let pose_id = submit_pose(1, &Pose { vertices: vec![] });
    eprintln!("https://poses.live/solutions/{}", pose_id);
}

#[derive(serde::Deserialize)]
struct SubmitResponse {
    id: String,
}

pub fn submit_pose(problem_id: i32, pose: &Pose) -> String {
    let resp = ureq::post(&format!("https://poses.live/api/problems/{}/solutions", problem_id))
        .set("Authorization", &format!("Bearer {}", API_KEY))
        .send_bytes(&serde_json::to_vec(pose).unwrap())
        .unwrap();
    assert_eq!(resp.status(), 200, "{:?}", resp);
    let resp = resp.into_string().unwrap();
    let resp: SubmitResponse = serde_json::from_str(&resp).unwrap();
    resp.id
}

crate::entry_point!("scrape_poses", scrape_poses, _EP3);
fn scrape_poses() {
    let agent = ureq::agent();
    let page = agent.post("https://poses.live/login")
         .set("Content-Type", "application/x-www-form-urlencoded")
         .send_string("login.email=jm%40memorici.de&login.password=uy2c92JKQAtSRfb").unwrap()
         .into_string().unwrap();
    println!("{}", page);
    scrape_one_problem(&agent, 1);
}

fn scrape_one_problem(agent: &ureq::Agent, n: i32) {
    let page = agent.get(&format!("https://poses.live/problems/{}", n))
         .call().unwrap().into_string().unwrap();
    println!("{}", page);
    // TODO: scrape html table from it
}
