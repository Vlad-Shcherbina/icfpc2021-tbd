// A client library for their portal https://poses.live

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

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

crate::entry_point!("scrape_poses_demo", scrape_poses_demo, _EP3);
fn scrape_poses_demo() {
    let mut scraper = Scraper::new();
    let pi = scraper.problem_info(1);
    dbg!(&pi);
    dbg!(pi.highscore());
    dbg!(pi.latest());
    let sol = scraper.get_pose_by_id("b9a6c73d-dd2c-4a54-9f6e-75ea51d11c79");
    dbg!(&sol);
}

crate::entry_point!("scrape_cache", scrape_cache, _EP4);
fn scrape_cache() {
    const COUNT: i32 = 132;  // 132
    let mut cache = match File::open(project_path("cache/server.cache")) {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            serde_json::from_str(&buf).unwrap()
        },
        Err(_) => ProblemCache { problems: HashMap::new(), poses: HashMap::new() },
    };
    cache.problems = HashMap::new();

    let mut scraper = Scraper::new();
    for i in 1..=COUNT {
        eprintln!("problem {}...", i);
        let p = scraper.problem_info(i).clone();
        cache.problems.insert(i, p.clone());
        for ps in &p.poses {
            if cache.poses.contains_key(&ps.id) { continue };
            eprintln!("pose {}...", &ps.id);
            let pose = scraper.get_pose_by_id(&ps.id);
            if let None = pose { continue; }
            let pci = PoseCacheItem{ pose: pose.unwrap(), eval: ps.er };
            cache.poses.insert(ps.id.clone(), pci);
        }
    }    
    let mut file = File::create(project_path("cache/server.cache")).unwrap();
    file.write_all(&serde_json::to_vec(&cache).unwrap()).unwrap();
}

pub fn read_cache() -> ProblemCache {
    let mut file = File::open(project_path("cache/server.cache")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    serde_json::from_str(&content).unwrap()
}

pub struct Scraper {
    agent: ureq::Agent,
    global_highscore_page: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone, Copy)]
pub enum EvaluationResult {
    Pending,  // hourglass
    Invalid,  // cross
    Valid { dislikes: i64 },
}

impl std::fmt::Display for EvaluationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationResult::Pending => write!(f, "⏳"),
            EvaluationResult::Invalid => write!(f, "❌"),
            EvaluationResult::Valid { dislikes } => write!(f, "{}", dislikes),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct PoseInfo {
    pub id: String,
    pub er: EvaluationResult,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct ProblemInfo {
    poses: Vec<PoseInfo>,
    global_highscore: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug)]
pub struct PoseCacheItem {
    pub pose: Pose,
    pub eval: EvaluationResult,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug)]
pub struct ProblemCache {
    pub problems: HashMap<i32, ProblemInfo>,
    pub poses: HashMap<String, PoseCacheItem>,
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

    pub fn latest(&self) -> Option<&PoseInfo> {
        if self.poses.is_empty() {
            None
        } else {
            Some(&self.poses[0])
        }
    }
}

impl Scraper {
    pub fn new() -> Self {
        let agent = ureq::agent();
        let _page = agent.post("https://poses.live/login")
             .set("Content-Type", "application/x-www-form-urlencoded")
             .send_string("login.email=jm%40memorici.de&login.password=uy2c92JKQAtSRfb").unwrap()
             .into_string().unwrap();
        Scraper { agent, global_highscore_page: None }
    }

    pub fn problem_info(&mut self, problem_id: i32) -> ProblemInfo {
        let page = self.agent.get(&format!("https://poses.live/problems/{}", problem_id))
            .call().unwrap().into_string().unwrap();

        // yes, we use regex to parse HTML
        let re = regex::Regex::new("<tr><td><a href=\"/solutions/(.*?)\".*?</a></td><td>(.*?)</td></tr>").unwrap();

        let mut poses = vec![];
        for pose in re.captures_iter(&page) {
            let id = pose.get(1).unwrap().as_str().to_string();
            match pose.get(2).unwrap().as_str() {
                "❌" => poses.push( PoseInfo {
                    id,
                    er: EvaluationResult::Invalid,
                }),
                "⏳" => poses.push( PoseInfo {
                    id,
                    er: EvaluationResult::Pending,
                }),
                other => poses.push( PoseInfo {
                    id,
                    er: EvaluationResult::Valid {
                        dislikes: other.parse().unwrap(),
                    },
                }),
            };
        }
        ProblemInfo { poses, global_highscore: self.get_global_highscore(problem_id) }
    }

    pub fn get_global_highscore(&mut self, problem_id: i32) -> i32 {
        let agent = &mut self.agent;
        let page = self.global_highscore_page.get_or_insert_with(|| {
            agent.get("https://poses.live/problems")
                .call().unwrap().into_string().unwrap()
        });

        let re = regex::Regex::new(&format!("<tr><td><a href=\"/problems/{}\">{}</a></td><td>.*?</td><td>(\\d+)</td></tr>", problem_id, problem_id)).unwrap();

        re.captures(page).unwrap().get(1).unwrap().as_str().parse::<i32>().unwrap()
    }

    pub fn get_pose_by_id(&mut self, pose_id: &str) -> Option<Pose> {
        let data = self.agent.get(&format!("https://poses.live/solutions/{}/download", pose_id))
            .call();

        match data {
            Ok(d) => match serde_json::from_str(&d.into_string().unwrap()) {
                Ok(a) => Some(a),                
                Err(msg) => {
                    eprintln!("Error {} in {}", msg, pose_id);
                    None
                }
            },
            Err(ureq::Error::Status(404, _)) => {
                None
            }
            Err(ureq::Error::Status(500, _)) => {
                None
            }
            r => panic!("{:?}", r),
        }
    }
}
