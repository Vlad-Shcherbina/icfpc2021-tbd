use rand::Rng;
//use rand::seq::SliceRandom;
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

crate::entry_point!("rail_constrained", rail_constrained);
fn rail_constrained() {
    let problem_id: i32 = std::env::args().nth(2).unwrap().parse().unwrap();

    let mut submitter = Submitter::new(problem_id, "rail_constrained".to_string());

    
    let p = load_problem(problem_id);
    let bonuses = vec![];

    let mut checker = Checker::new(&p, &bonuses, p.figure.vertices.len());
    let edges = checker.edges.clone();
    let mut inci: Vec<Vec<usize>> = vec![vec![]; edges.len()];
    for (i, &(start, end)) in edges.iter().enumerate() {
        inci[start].push(i);
        inci[end].push(i);
    }

    let mut rng = rand::thread_rng();

    //eprintln!("deltass");
    let deltass: Vec<Vec<Pt>> = checker.edge_ranges.iter()
        .map(|&(min_d, max_d, _)| deltas(min_d, max_d))
        .collect();

    'outer: loop {
        //eprintln!("------");
        let mut pts: Vec<Option<Pt>> = vec![None; p.figure.vertices.len()];
        let mut placements: Vec<Vec<Pt>> = vec![vec![]; p.figure.vertices.len()];

        let mut corners_filled: Vec<bool> = vec![false; p.hole.len()];

        loop {
            //eprintln!("Loop");
            let v_idx;
            let pt; 
            // Choose most constrained corner and try to fill it.
            let unfilled_corners: Vec<usize> = (1..corners_filled.len()).filter(|i| !corners_filled[*i]).collect();
            let mut corner_placements: Vec<Vec<usize>> = vec![vec![]; p.hole.len()];

            // TODO: this is n^2
            for corner in unfilled_corners {
                //dbg!(corner);
                for (vertice_idx, _) in pts.iter().enumerate().filter(|(_, pt)| pt.is_none()) {
                    // Those that are unconstrained or those that contain corner.
                    //dbg!(vertice_idx, &placements[vertice_idx]);
                    if placements[vertice_idx].is_empty()
                        || placements[vertice_idx].iter().any(|point| *point == p.hole[corner]) {
                            corner_placements[corner].push(vertice_idx);
                    }
                }
            }
            //dbg!(&corner_placements);
            let min_corner_placement_size = corner_placements.iter()
                .map(|v| v.len()).filter(|v| *v > 0).min().unwrap_or(0);
            //eprintln!("{}", min_corner_placement_size);
            if min_corner_placement_size != 0 {
                let most_constrained_corners: Vec<usize> = corner_placements.iter().enumerate()
                    .filter(|(_, v)| v.len() == min_corner_placement_size).map(|(i, _)| i).collect();
                let corner = most_constrained_corners[rng.gen_range(0..most_constrained_corners.len())];
                v_idx = corner_placements[corner][rng.gen_range(0..corner_placements[corner].len())];
                corners_filled[corner] = true;
                pt = p.hole[corner];
                //eprintln!("Added corner");
            } else {
                //eprintln!("Adding noncorner");
                // place a non-corner vertex
            
                // Select most constrained vertice to place next.
                let qq = pts.iter().zip(placements.iter()).enumerate()
                .filter(|(_i, (pt, placement))| {
                    pt.is_none() && !placement.is_empty()
                })
                .min_by_key(|(_i, (_, placement))| placement.len());

                // Place it randomly within constraints.
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
            }

            pts[v_idx] = Some(pt);

            // Update placement constraints for each unplaced neighbor.
            for &e_idx in &inci[v_idx] {
                let v2_idx = edges[e_idx].0 + edges[e_idx].1 - v_idx;
                if pts[v2_idx].is_some() {
                    continue;
                }

                let (min_d, max_d, _) = checker.edge_ranges[e_idx];
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
            // Update constraints for corners.
            //TODO

            // If all vertices placed, evaluate pose.
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
