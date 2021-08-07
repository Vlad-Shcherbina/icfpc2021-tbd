#![allow(unused_imports)]

use crate::checker::{check_pose, get_dislikes, list_unlocked_bonuses};
use crate::domain_model::{UnlockedBonus, BonusName};
use crate::prelude::*;
use crate::poses_live::submit_pose;
use crate::poses_live::{Scraper, EvaluationResult};
use crate::db::{connect, get_solutions_stats_by_problem};

#[derive(Debug, Clone)]
pub struct Rank {
    pub used_bonus: Option<BonusName>,
    pub dislikes: i64,
    pub unlocked_bonuses: Vec<UnlockedBonus>,
}

impl Rank {
    pub fn new(p: &Problem, pose: &Pose) -> Rank {
        Rank {
            used_bonus: match &pose.bonuses[..] {
                [] => None,
                [x] => Some(x.bonus),
                _ => panic!(),
            },
            dislikes: get_dislikes(p, &pose.vertices),
            unlocked_bonuses: list_unlocked_bonuses(p, &pose.vertices),
        }
    }

    pub fn from_db(s: &crate::db::SolutionStats) -> Rank {
        Rank {
            used_bonus: s.bonus_used,
            dislikes: s.dislikes,
            unlocked_bonuses: s.bonuses_unlocked.clone(),
        }
    }

    // less is better
    fn sort_key(&self) -> i64 {
        self.dislikes 
            - self.unlocked_bonuses.len() as i64 
            + if self.used_bonus == None { 0 } else { 1 }
    }

    pub fn dominates(&self, other: &Rank) -> bool {
        if other.dislikes < self.dislikes {
            return false;
        }
        for b in &self.used_bonus {
            if !other.used_bonus.iter().any(|q| q == b) {
                return false;
            }
        }
        /*for ub in &other.unlocked_bonuses {
            if !self.unlocked_bonuses.iter().any(|q| q == ub) {
                return false;
            }
        }*/

        true
    }
}

pub struct Submitter {
    problem_id :i32,
    // last_attempt: std::time::Instant,
    db_client: postgres::Client,
    front: Vec<(Rank, Option<Pose>)>,
    solver: String,
}

pub fn pareto<T>(mut front: Vec<(Rank, T)>) -> Vec<(Rank, T)> {
    front.sort_by_key(|(r, _)| r.sort_key());
    let mut res: Vec<(Rank, T)> = vec![];
    for (q, t) in front {
        if res.iter().any(|(qq, _)| qq.dominates(&q)) {
            continue;
        }
        res.push((q, t));
    }
    res
}

impl Submitter {
    pub fn new(problem_id: i32, solver: String) -> Submitter {
        let mut db_client = connect().unwrap();
        
        let stats = get_solutions_stats_by_problem(&mut db_client, problem_id).unwrap();

        let mut front: Vec<(Rank, Option<Pose>)> = stats
            .iter()
            .map(|s| (Rank::from_db(s), None))
            .collect();
        for q in &front {
            eprintln!("{:?}", q.0);
        }
        eprintln!("--");
        front = pareto(front);
        for q in &front {
            eprintln!("{:?}", q.0);
        }

        Submitter {
            problem_id,
            db_client,
            front,
            solver: solver,
        }
    }

    pub fn update(&mut self, p: &Problem, pose: &Pose) {
        let rank = Rank::new(p, pose);

        if self.front.iter().any(|(r, _)| r.dominates(&rank)) {
            return;
        }

        eprintln!("found improvement {:?}, submitting to db", rank);
        let solution_id = crate::db::write_valid_solution_to_db(
            &mut self.db_client,
            self.problem_id,
            pose,
            rank.dislikes,
            &self.solver
        ).unwrap();
        eprintln!("\nSee solution at http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}\n",
            self.problem_id, solution_id);

        self.front.push((rank, Some(pose.clone())));
        self.front = pareto(self.front.clone());

        eprintln!("--");
        for q in &self.front {
            eprintln!("{:?}", q.0);
        }
        eprintln!("--");
    }
}
