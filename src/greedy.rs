#![allow(unused_imports)]

use rand::Rng;
use crate::checker::{length_range, check_pose};
use crate::geom::*;
use crate::prelude::*;
use crate::graph::*;
use crate::shake::ShakeRequest;
use rand::prelude::SliceRandom;

fn orig_distance(problem: &Problem, v1_id: usize, v2_id: usize) -> i64 {
    problem.figure.vertices[v1_id].dist2(problem.figure.vertices[v2_id])
}

fn deformation_limits(problem: &Problem, v1_id: usize, v2_id: usize) -> (i64, i64) {
    let orig_d2 = orig_distance(problem, v1_id, v2_id);
    return crate::checker::length_range(orig_d2, problem.epsilon);
}


fn valid_positions(problem: &Problem, vs: &mut Vec<Pt>, idx: usize) -> Vec<Pt> {
    let neighbours: Vec<_> = neighbours(&problem.figure.edges, idx).collect();
    let (pt_min, pt_max) = bounding_box(&problem.hole).unwrap();
    let mut result = vec!{};
    for x in pt_min.x..=pt_max.x {
        'l: for y in pt_min.y..=pt_max.y {
            let assumed_pos = Pt { x, y };
            for neighbour_idx in &neighbours {
                let neighbour = vs[*neighbour_idx];
                let (min_dist, max_dist) = deformation_limits(&problem, idx, *neighbour_idx);
                let new_dist = assumed_pos.dist2(neighbour);
                if !(min_dist <= new_dist && new_dist <= max_dist) {
                    // eprintln!("Drop {:?} by distance", assumed_pos);
                    continue 'l;
                }
                if !segment_in_poly((neighbour, assumed_pos), &problem.hole) {
                    // eprintln!("Drop {:?} by intersection", assumed_pos);
                    continue 'l;
                }
            }
            result.push(assumed_pos);
        }
    }
    return result;
}


pub fn greedy_shake(r: &ShakeRequest) -> Vec<Pt> {
    let mut selected = r.selected.clone();
    if selected.iter().all(|&s| !s) {
        selected = vec![true; selected.len()];
    }
    let mut selected_idxs = vec![];
    for (i, &sel) in selected.iter().enumerate() {
        if sel {
            selected_idxs.push(i);
        }
    }
    let mut rng = rand::thread_rng();

    let expand = |vs: &mut Vec<Pt>| {
        let pr = check_pose(&r.problem, &Pose { vertices: vs.clone(), bonuses: vec![] });
        assert!(pr.valid);
        let mut prev_dislikes = pr.dislikes;
        let mut cur_dislikes = pr.dislikes;
        loop {
            //dbg!(cur_dislikes);
            for idx in selected_idxs.iter() {
                //dbg!(idx);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 { continue };
                        let cur = vs[*idx];
                        vs[*idx] = Pt{x: cur.x + dx, y: cur.y + dy};
                        let pr = check_pose(&r.problem, &Pose { vertices: vs.clone(), bonuses: vec![] });
                        if pr.valid && pr.dislikes < cur_dislikes {
                            cur_dislikes = pr.dislikes;
                        } else {
                            vs[*idx] = cur;
                        }
                    }
                }
            }
            if cur_dislikes == prev_dislikes {
                break
            } else {
                prev_dislikes = cur_dislikes;
            }
        }
    };

    let shake = |vs: &mut Vec<Pt>, rng:  &mut dyn rand::RngCore| -> i64 {
        let pr = check_pose(&r.problem, &Pose { vertices: vs.clone(), bonuses: vec![] });
        assert!(pr.valid);
        let mut cur_dislikes = pr.dislikes;
        for _ in 0..1 {
            for idx in selected_idxs.iter() {
                //dbg!(idx);
                //let dx = rng.gen_range(-1..=1);
                //let dy = rng.gen_range(-1..=1);
                //if dx == 0 && dy == 0 { continue };
                //let new_pos = Pt{x: cur.x + dx, y: cur.y + dy};
                let perturbations = valid_positions(&r.problem, vs, *idx);
                if perturbations.is_empty() {
                    continue;
                }
                let new_pos = perturbations.choose(rng).unwrap();
                let cur = vs[*idx];
                vs[*idx] = *new_pos;
                let pr = check_pose(&r.problem, &Pose { vertices: vs.clone(), bonuses: vec![] });
                // Here we allow perturbations that don't reduce dislikes.
                if pr.valid && pr.dislikes <= cur_dislikes {
                    cur_dislikes = pr.dislikes;
                } else {
                    vs[*idx] = cur;
                }
            }
        }
        return cur_dislikes;
    };

    let mut cur_vs = r.vertices.clone();

    let pr = check_pose(&r.problem, &Pose { vertices: cur_vs.clone(), bonuses: vec![] });
    if !pr.valid {
        dbg!("invalid pose passed to greedy shake");
        return cur_vs;
    }
    let mut dislikes = pr.dislikes;
    let convergence_cutoff = r.param*100;
    let mut i = 0;
    loop {
        //dbg!(i);
        expand(&mut cur_vs);
        //dbg!("Shake");
        let cur_dislikes = shake(&mut cur_vs, &mut rng);
        if cur_dislikes < dislikes {
            dislikes = cur_dislikes;
            i = 0;
        } else {
            i += 1;
            if i > convergence_cutoff {
                break;
            }
        }
    }

    cur_vs
}
