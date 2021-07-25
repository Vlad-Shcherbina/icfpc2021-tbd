use std::collections::HashMap;
use crate::checker::check_pose;
use crate::prelude::*;
use crate::poses_live::Scraper;
use crate::submitter::{Rank, pareto};

crate::entry_point!("planner", planner);
fn planner() {
    // let all_probs = || all_problem_ids();  // TODO
    let all_probs = || 1..20;

    let mut scraper = Scraper::new();
    let mut fronts = HashMap::new();
    let mut latest = HashMap::new();
    for problem_id in all_probs() {
        dbg!(problem_id);
        let problem = load_problem(problem_id);
        let pi = scraper.problem_info(problem_id);
        let mut front: Vec<(Rank, (String, Pose))> = vec![];
        for pp in &pi.poses {
            let pose = scraper.get_pose_by_id(&pp.id);
            let pose = match pose {
                Some(p) => p,
                None => continue,  // TODO: this shit is not defensive enough
            };
            // dbg!(&pp.id);
            let cpr = check_pose(&problem, &pose);
            // eprintln!("ok");
            if cpr.valid {
                front.push((Rank::new(&problem, &pose), (pp.id.clone(), pose)));
            }
        }
        front = pareto(front);
        fronts.insert(problem_id, front);

        let p = scraper.get_pose_by_id(&pi.latest().unwrap().id);
        if let Some(p) = p {
            latest.insert(problem_id, p);
        }
        // let pi = scraper.latest;
    }

    let mut finl = HashMap::new();
    let mut bonuses: HashMap<i32, Vec<PoseBonus>> = HashMap::new();

    for problem_id in all_probs() {
        if finl.contains_key(&problem_id) {
            continue;
        }
        let front = &fronts[&problem_id];
        if front.len() == 1 {
            let q = front[0].clone();
            eprintln!("singleton: {} {:?}", problem_id, q.0);
            for b in &q.0.unlocked_bonuses {
                bonuses.entry(b.problem_id).or_default().push(PoseBonus {
                    bonus: b.name,
                    problem: problem_id,
                    edge: None,
                });
            }
            finl.insert(problem_id, q);
        }
    }
    /*for (problem_id, front) in fronts.iter_mut() {
        *front = pareto(front.clone().into_iter().map(|mut q| {
            let used_bonuses = &mut q.0.used_bonuses;
        }).collect());
    }*/
    for problem_id in all_probs() {
        if finl.contains_key(&problem_id) {
            continue;
        }
        // for q in &fronts[&problem_id]
    }

    // for prob
}