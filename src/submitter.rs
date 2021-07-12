use crate::checker::{check_unlocked, get_dislikes, list_unlocked_bonuses};
use crate::domain_model::{BonusName, UnlockedBonus};
use crate::prelude::*;
use crate::poses_live::submit_pose;
use crate::poses_live::{Scraper, PoseInfo, EvaluationResult};

#[derive(Debug, Clone)]
struct Rank {
    used_bonuses: Vec<PoseBonus>,
    dislikes: i64,
    unlocked_bonuses: Vec<UnlockedBonus>,
}

impl Rank {
    fn new(p: &Problem, pose: &Pose) -> Rank {
        Rank {
            used_bonuses: vec![],  // TODO
            dislikes: get_dislikes(&p, &pose.vertices),
            unlocked_bonuses: list_unlocked_bonuses(p, &pose.vertices),
        }
    }

    // less is better
    fn sort_key(&self) -> i64 {
        self.dislikes - self.unlocked_bonuses.len() as i64 + self.used_bonuses.len() as i64
    }

    fn dominates(&self, other: &Rank) -> bool {
        if other.dislikes < self.dislikes {
            return false;
        }
        // TODO: used_bonuses
        for ub in &other.unlocked_bonuses {
            if !self.unlocked_bonuses.iter().any(|q| q == ub) {
                return false;
            }
        }

        true
    }
}

pub struct Submitter {
    problem_id :i32,
    best_dislikes: i64,
    to_submit: Option<(i64, Pose)>,
    last_attempt: std::time::Instant,
    front: Vec<(Rank, Option<Pose>)>,
}

fn pareto<T>(mut front: Vec<(Rank, T)>) -> Vec<(Rank, T)> {
    front.sort_by_key(|(r, _)| r.sort_key());
    let mut res: Vec<(Rank, T)> = vec![];
    for (q, t) in front {
        if res.iter().any(|(qq, _)| qq.dominates(&q)) {
            continue;
        }
        res.push((q, t));
    }
    return res
}

impl Submitter {
    pub fn new(problem_id: i32) -> Submitter {
        let problem = load_problem(problem_id);
        let mut scraper = Scraper::new();
        let pi = scraper.problem_info(problem_id);

        let mut front: Vec<(Rank, Option<Pose>)> = vec![];
        for pp in &pi.poses {
            match pp.er {
                EvaluationResult::Pending => {}
                EvaluationResult::Invalid => {}
                EvaluationResult::Valid { dislikes } => {
                    let pose = scraper.get_pose_by_id(&pp.id).unwrap();
                    if pose.bonuses.is_empty() {
                        front.push((Rank::new(&problem, &pose), None));
                    }
                }
            }
        }
        for q in &front {
            eprintln!("{:?}", q.0);
        }
        eprintln!("--");
        front = pareto(front);
        for q in &front {
            eprintln!("{:?}", q.0);
        }

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
            last_attempt,
            front,
        }
    }
    pub fn update(&mut self, p: &Problem, pose: &Pose) {
        let rank = Rank::new(p, pose);

        if self.front.iter().any(|(r, _)| r.dominates(&rank)) {
            return;
        }

        eprintln!("found improvement {:?}, will try to submit soon", rank);

        self.front.push((rank, Some(pose.clone())));
        self.front = pareto(self.front.clone());

        eprintln!("--");
        for q in &self.front {
            eprintln!("{:?}", q.0);
        }
        eprintln!("--");
    }
    pub fn try_submit(&mut self) -> bool {
        if self.best_dislikes == 0 && self.to_submit.is_none() {
            eprintln!("nothing to do, optimal solution found and submitted");
            return true;
        }

        let q = self.front.iter_mut().find(|(_, to_submit)| to_submit.is_some());
        if let Some((rank, pose)) = q{
            if self.last_attempt.elapsed().as_secs_f64() > 30.0 {
                match submit_pose(self.problem_id, pose.as_ref().unwrap()) {
                    Ok(pose_id) => {
                        eprintln!("{:?} submitted https://poses.live/solutions/{}", rank, pose_id);
                        *pose = None;
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
