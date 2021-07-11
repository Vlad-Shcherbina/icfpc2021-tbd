use rand::Rng;
use crate::prelude::*;
use crate::checker::Checker;
use crate::poses_live::submit_pose;

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

    'outer: loop {
        eprintln!("------");
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
                    for delta in deltas(min_d, max_d) {
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
                    eprintln!("deadend");
                    continue 'outer;
                }
            }

            dbg!(v_idx);
            dbg!(pt);

            /*for i in 0..pts.len() {
                match pts[i] {
                    Some(pt) => eprintln!("{}: {:?}", i, pt),
                    None => eprintln!("{}: {:?}", i, placements[i]),
                }
            }*/

            if pts.iter().all(|pt| pt.is_some()) {
                eprintln!("solved");
                let pose = Pose {
                    vertices: pts.iter().map(|pt| pt.unwrap()).collect(),
                    bonuses,
                };
                // todo!();
                match submit_pose(problem_id, &pose) {
                    Ok(pose_id) => {
                        eprintln!("submitted https://poses.live/solutions/{}", pose_id)
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
                return;
            }
        }
    }
}
