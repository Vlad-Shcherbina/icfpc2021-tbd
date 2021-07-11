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
    let mut scraper = Scraper::new();
    let pi = scraper.problem_info(1);
    dbg!(&pi);
    dbg!(pi.highscore());
    let sol = scraper.get_pose_by_id("b9a6c73d-dd2c-4a54-9f6e-75ea51d11c79".to_string());
    dbg!(&sol);
}

pub struct Scraper {
    agent: ureq::Agent,
}

#[derive(Debug)]
#[allow(dead_code)]   // TODO: remove when scraper is done
pub enum EvaluationResult {
    Pending,  // hourglass
    Invalid,  // cross
    Valid { dislikes: i64 },
}

#[derive(Debug)]
pub struct PoseInfo {
    pub id: String,
    pub er: EvaluationResult,
}

#[derive(Debug)]
pub struct ProblemInfo {
    poses: Vec<PoseInfo>,
    global_highscore: i32,
}

impl ProblemInfo {
    pub fn highscore(&self) -> Option<&PoseInfo> {
        self.poses.iter()
        .filter_map(|pi| match pi.er {
            EvaluationResult::Valid { dislikes } => Some((dislikes, pi)),
            EvaluationResult::Invalid => None,
            EvaluationResult::Pending => None,
        })
        .min_by_key(|q| q.0)
        .map(|q| q.1)
    }

    pub fn _latest(&self) -> Option<&PoseInfo> {
        // can't be implemented right now because we only have valid submissions
        todo!()
    }
}

impl Scraper {
    pub fn new() -> Self {
        let agent = ureq::agent();
        let _page = agent.post("https://poses.live/login")
             .set("Content-Type", "application/x-www-form-urlencoded")
             .send_string("login.email=jm%40memorici.de&login.password=uy2c92JKQAtSRfb").unwrap()
             .into_string().unwrap();
        Scraper { agent }
    }

    pub fn problem_info(&mut self, problem_id: i32) -> ProblemInfo {
        let page = self.agent.get(&format!("https://poses.live/problems/{}", problem_id))
            .call().unwrap().into_string().unwrap();

        // yes, we use regex to parse HTML
        let re = regex::Regex::new("<tr><td><a href=\"/solutions/(.*?)\".*?</a></td><td>(\\d+)</td></tr>").unwrap();

        // TODO: detect crosses and hourglasses, not only valid solutions
        let mut poses = vec![];
        for pose in re.captures_iter(&page) {
            poses.push( PoseInfo {
                id: pose.get(1).unwrap().as_str().to_string(),
                er: EvaluationResult::Valid {
                    dislikes: pose.get(2).unwrap().as_str().parse().unwrap(),
                },
            });
        }
        ProblemInfo { poses, global_highscore: self.get_global_highscore(problem_id) }
    }

    pub fn get_global_highscore(&mut self, problem_id: i32) -> i32 {
        let page = self.agent.get("https://poses.live/problems")
            .call().unwrap().into_string().unwrap();

        let re = regex::Regex::new(&format!("<tr><td><a href=\"/problems/{}\">{}</a></td><td>.*?</td><td>(\\d+)</td></tr>", problem_id, problem_id)).unwrap();

        re.captures(&page).unwrap().get(1).unwrap().as_str().parse::<i32>().unwrap()
    }

    pub fn get_pose_by_id(&mut self, pose_id: String) -> Option<Pose> {
        let data = self.agent.get(&format!("https://poses.live/solutions/{}/download", pose_id))
            .call();

        match data {
            Ok(data) => {
                Some(serde_json::from_str(&data.into_string().unwrap()).unwrap())
            }
            Err(ureq::Error::Status(500, _)) => {
                None
            }
            r => panic!("{:?}", r),
        }


    }
}
