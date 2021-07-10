// A client library for their portal https://poses.live

use std::collections::HashMap;

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
    match submit_pose(1, &Pose { vertices: vec![], bonuses: vec![] }) {
        Ok(pose_id) => eprintln!("https://poses.live/solutions/{}", pose_id),
        Err(e) => eprintln!("{}", e),
    }
}

#[derive(serde::Deserialize)]
struct SubmitResponse {
    id: String,
}

pub fn submit_pose(problem_id: i32, pose: &Pose) -> Result<String, String> {
    let resp = ureq::post(&format!("https://poses.live/api/problems/{}/solutions", problem_id))
        .set("Authorization", &format!("Bearer {}", API_KEY))
        .send_bytes(&serde_json::to_vec(pose).unwrap());
    match resp {
        Ok(resp) => {
            assert_eq!(resp.status(), 200, "{:?}", resp);
            let resp = resp.into_string().unwrap();
            let resp: SubmitResponse = serde_json::from_str(&resp).unwrap();
            Ok(resp.id)
        }
        Err(ureq::Error::Status(429, resp)) => {
            let resp = resp.into_string().unwrap();
            Err(resp)
        }
        r => panic!("{:?}", r),
    }
}

crate::entry_point!("scrape_poses", scrape_poses, _EP3);
fn scrape_poses() {
    let agent = ureq::agent();
    let _page = agent.post("https://poses.live/login")
         .set("Content-Type", "application/x-www-form-urlencoded")
         .send_string("login.email=jm%40memorici.de&login.password=uy2c92JKQAtSRfb").unwrap()
         .into_string().unwrap();
    scrape_one_problem(&agent, 1);
}

pub fn scrape_problem_n(n: i32) -> HashMap<String, i32> {
    let agent = ureq::agent();
    let _page = agent.post("https://poses.live/login")
         .set("Content-Type", "application/x-www-form-urlencoded")
         .send_string("login.email=jm%40memorici.de&login.password=uy2c92JKQAtSRfb").unwrap()
         .into_string().unwrap();

    scrape_one_problem(&agent, n)
}


fn scrape_one_problem(agent: &ureq::Agent, n: i32) -> HashMap<String, i32> {
    let page = agent.get(&format!("https://poses.live/problems/{}", n))
         .call().unwrap().into_string().unwrap();

    let mut poses = HashMap::new();

    // yes, we use regex to parse HTML
    let re = regex::Regex::new("<tr><td><a href=\"/solutions/(.*?)\".*?</a></td><td>(\\d+)</td></tr>").unwrap();
    for pose in re.captures_iter(&page) {
        poses.insert(
            pose.get(1).unwrap().as_str().to_string(),
            pose.get(2).unwrap().as_str().parse::<i32>().unwrap()
        );
    }

    poses

    //for (id,dislikes) in &poses {
    //    println!("{}, {}", id, dislikes);
    //}

}
