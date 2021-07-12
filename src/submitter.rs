use crate::prelude::*;
use crate::poses_live::submit_pose;
use crate::poses_live::{Scraper, PoseInfo, EvaluationResult};

pub struct Submitter {
    problem_id :i32,
    best_dislikes: i64,
    to_submit: Option<(i64, Pose)>,
    last_attempt: std::time::Instant
}

impl Submitter {
    pub fn new(problem_id: i32) -> Submitter {
        let mut scraper = Scraper::new();
        let pi = scraper.problem_info(problem_id);

        let best_dislikes = match pi.highscore() {
            Some(PoseInfo { er: EvaluationResult::Valid { dislikes }, .. }) => *dislikes,
            _ => 1_000_000_000,
        };
        eprintln!("best dislikes so far: {}", best_dislikes);
        let last_attempt = std::time::Instant::now();
        Submitter {
            problem_id,
            best_dislikes,
            to_submit: None,
            last_attempt
        }
    }
    pub fn update(&mut self, dislikes: i64, pose: &Pose) {
        if self.best_dislikes > dislikes {
            eprintln!("solved, {} dislikes", dislikes);
            eprintln!("FOUND IMPROVEMENT, will try to submit soon");
            self.best_dislikes = dislikes;
            self.to_submit = Some((dislikes, pose.clone()));
        }

    }
    pub fn try_submit(&mut self) -> bool {
        if self.best_dislikes == 0 && self.to_submit.is_none() {
            eprintln!("nothing to do, optimal solution found and submitted");
            return true;
        }

        if let Some((dislikes, pose)) = self.to_submit.as_ref() {
            if self.last_attempt.elapsed().as_secs_f64() > 30.0 {
                match submit_pose(self.problem_id, pose) {
                    Ok(pose_id) => {
                        eprintln!("(dislikes: {}) submitted https://poses.live/solutions/{}", dislikes, pose_id);
                        self.to_submit = None;
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
                self.last_attempt = std::time::Instant::now();
            }
        }
        false
    }
}
