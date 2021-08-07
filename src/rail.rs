use rand::Rng;
use crate::domain_model::BonusName;
use crate::geom::{bounding_box, pt_in_poly};
use crate::prelude::*;
use crate::checker::Checker;
use crate::submitter::Submitter;

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

    let mut superflex = false;
    match std::env::args().nth(3).as_deref() {
        Some("S") => superflex = true,
        None => {},
        _ => panic!()
    };

    let mut submitter = Submitter::new(problem_id, "rail".to_string());

    let p = load_problem(problem_id);
    let mut bonuses = vec![];
    if superflex {
        bonuses.push(PoseBonus {
            bonus: BonusName::SUPERFLEX,
            problem: 4242,
            edge: None,
        });
    }
    dbg!(&bonuses);

    let mut checker = Checker::new(&p, &bonuses, p.figure.vertices.len());
    let edges = checker.edges.clone();
    let mut inci: Vec<Vec<usize>> = vec![vec![]; edges.len()];
    for (i, &(start, end)) in edges.iter().enumerate() {
        inci[start].push(i);
        inci[end].push(i);
    }

    let (pt_min, pt_max) = bounding_box(&p.hole).unwrap();
    let mut inside = vec![];
    for x in pt_min.x..=pt_max.x {
        for y in pt_min.y..=pt_max.y {
            let pt = Pt::new(x, y);
            if pt_in_poly(pt, &p.hole) {
                inside.push(pt);
            }
        }
    }

    let mut rng = rand::thread_rng();

    let deltass: Vec<Vec<Pt>> = checker.edge_ranges.iter()
        .map(|&(min_d, max_d, _)| deltas(min_d, max_d))
        .collect();

    'outer: loop {

        // eprintln!("------");
        let mut pts: Vec<Option<Pt>> = vec![None; p.figure.vertices.len()];
        let mut placements: Vec<Vec<Pt>> = vec![vec![]; p.figure.vertices.len()];

        let superflex_e_idx = if superflex {
            Some(rng.gen_range(0..p.figure.edges.len()))
        } else {
            None
        };

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
                    if rng.gen() {
                        pt = inside[rng.gen_range(0..inside.len())];
                    } else {
                        pt = p.hole[rng.gen_range(0..p.hole.len())];
                    }
                }
            }

            pts[v_idx] = Some(pt);

            for &e_idx in &inci[v_idx] {
                let v2_idx = edges[e_idx].0 + edges[e_idx].1 - v_idx;
                if pts[v2_idx].is_some() {
                    continue;
                }

                let (min_d, max_d, _) = checker.edge_ranges[e_idx];
                let placement = &mut placements[v2_idx];
                if placement.is_empty() {
                    if Some(e_idx) == superflex_e_idx {
                        for &pt2 in &inside {
                            if checker.edge_in_hole(pt, pt2) {
                                placement.push(pt2);
                            }
                        }
                    } else {
                        for &delta in &deltass[e_idx] {
                            let pt2 = pt + delta;
                            if checker.edge_in_hole(pt, pt2) {
                                placement.push(pt2);
                            }
                        }
                    }
                } else {
                    placement.retain(|&pt2| {
                        let d = pt.dist2(pt2);
                        (min_d <= d && d <= max_d || Some(e_idx) == superflex_e_idx)
                        && checker.edge_in_hole(pt, pt2)
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
                submitter.update(&p, &pose);
                continue 'outer;
            }
        }
    }
}
