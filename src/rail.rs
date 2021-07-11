use rand::Rng;
use crate::prelude::*;
use crate::checker::{Checker, get_dislikes};
use crate::poses_live::submit_pose;
use crate::poses_live::{Scraper, PoseInfo, EvaluationResult};

fn deltas(min_d: i64, max_d: i64) -> Vec<Pt> {
    let mut result = vec![];
    for x in -max_d..=max_d {
        for y in -max_d..=max_d {
            let d = Pt { x, y }.len2();
            if min_d <= d && d <= max_d {
                result.push(Pt {x, y});
            }
        }
    }
    result
}

crate::entry_point!("rail", rail);
fn rail() {
    let problem_id: i32 = std::env::args().nth(2).unwrap().parse().unwrap();

    let mut scraper = Scraper::new();
    let pi = scraper.problem_info(problem_id);

    let mut best_dislikes = match pi.highscore() {
        Some(PoseInfo { er: EvaluationResult::Valid { dislikes }, .. }) => *dislikes,
        _ => 1000_000_000,
    };
    eprintln!("best dislikes so far: {}", best_dislikes);

    let mut to_submit: Option<(i64, Pose)> = None;
    let mut last_attempt = std::time::Instant::now();

    let p = load_problem(problem_id);
    let bonuses = vec![];

    let mut checker = Checker::new(&p, &bonuses);
    let edges = checker.edges.clone();
    let mut inci: Vec<Vec<usize>> = vec![vec![]; edges.len()];
    for (i, &(start, end)) in edges.iter().enumerate() {
        inci[start].push(i);
        inci[end].push(i);
    }

    let mut rng = rand::thread_rng();

    let deltass: Vec<Vec<Pt>> = checker.edge_ranges.iter()
        .map(|&(min_d, max_d)| deltas(min_d, max_d))
        .collect();

    'outer: loop {
        if best_dislikes == 0 && to_submit.is_none() {
            eprintln!("nothing to do, optimal solution found and submitted");
            return;
        }

        if let Some((dislikes, pose)) = to_submit.as_ref() {
            if last_attempt.elapsed().as_secs_f64() > 30.0 {
                match submit_pose(problem_id, &pose) {
                    Ok(pose_id) => {
                        eprintln!("(dislikes: {}) submitted https://poses.live/solutions/{}", dislikes, pose_id);
                        to_submit = None;
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
                last_attempt = std::time::Instant::now();
            }
        }

        // eprintln!("------");
        let mut pts: Vec<Option<Pt>> = vec![None; p.figure.vertices.len()];
        let mut placements: Vec<Vec<Pt>> = vec![vec![]; p.figure.vertices.len()];

        loop {
            let qq = pts.iter().zip(placements.iter()).enumerate()
            .filter(|(_i, (pt, placement))| {
                pt.is_none() && !placement.is_empty()
            })
            .min_by_key(|(_i, (_, placement))| placement.len());

            let v_idx;
            let pt;
            match qq {
                Some((i, (_, placement))) => {
                    v_idx = i;
                    pt = placement[rng.gen_range(0..placement.len())];
                },
                None => {
                    v_idx = rng.gen_range(0..pts.len());
                    pt = p.hole[rng.gen_range(0..p.hole.len())];
                }
            }

            pts[v_idx] = Some(pt);

            for &e_idx in &inci[v_idx] {
                let v2_idx = edges[e_idx].0 + edges[e_idx].1 - v_idx;
                if pts[v2_idx].is_some() {
                    continue;
                }

                let (min_d, max_d) = checker.edge_ranges[e_idx];
                let placement = &mut placements[v2_idx];
                if placement.is_empty() {
                    for &delta in &deltass[e_idx] {
                        let pt2 = pt + delta;
                        if checker.edge_in_hole(pt, pt2) {
                            placement.push(pt2);
                        }
                    }
                } else {
                    placement.retain(|&pt2| {
                        let d = pt.dist2(pt2);
                        min_d <= d && d <= max_d && checker.edge_in_hole(pt, pt2)
                    });
                }

                if placement.is_empty() {
                    // eprintln!("deadend");
                    continue 'outer;
                }
            }

            // dbg!(v_idx);
            // dbg!(pt);

            /*for i in 0..pts.len() {
                match pts[i] {
                    Some(pt) => eprintln!("{}: {:?}", i, pt),
                    None => eprintln!("{}: {:?}", i, placements[i]),
                }
            }*/

            if pts.iter().all(|pt| pt.is_some()) {
                // dbg!(checker.edge_cache.len());
                let pose = Pose {
                    vertices: pts.iter().map(|pt| pt.unwrap()).collect(),
                    bonuses: bonuses.clone(),
                };
                let dislikes = get_dislikes(&p, &pose.vertices);
                if dislikes < best_dislikes {
                    eprintln!("solved, {} dislikes", dislikes);
                    eprintln!("FOUND IMPROVEMENT, will try to submit soon");
                    best_dislikes = dislikes;
                    to_submit = Some((dislikes, pose));
                }
                continue 'outer;
            }
        }
    }
}
