#![allow(unused_imports)]

use rand::Rng;
use crate::checker::{length_range, check_pose, get_dislikes};
use crate::geom::*;
use crate::prelude::*;
use crate::graph::*;
use crate::shake::ShakeRequest;
use rand::prelude::SliceRandom;
use ndarray::Array2;

fn orig_distance(problem: &Problem, v1_id: usize, v2_id: usize) -> i64 {
    problem.figure.vertices[v1_id].dist2(problem.figure.vertices[v2_id])
}

fn deformation_limits(problem: &Problem, v1_id: usize, v2_id: usize) -> (i64, i64) {
    let orig_d2 = orig_distance(problem, v1_id, v2_id);
    crate::checker::length_range(orig_d2, problem.epsilon)
}


fn valid_positions(problem: &Problem, vs: &mut Vec<Pt>, idx: usize, hole_mask: &Array2<i16>) -> Vec<Pt> {
    let neighbours: Vec<_> = neighbours(&problem.figure.edges, idx).collect();
    let (pt_min, pt_max) = bounding_box(&problem.hole).unwrap();
    let mut result = vec![];
    for x in pt_min.x..=pt_max.x {
        'l: for y in pt_min.y..=pt_max.y {
            if hole_mask[[x as usize, y as usize]] != 0 {
                continue;
            }
            let assumed_pos = Pt { x, y };
            for neighbour_idx in &neighbours {
                let neighbour = vs[*neighbour_idx];
                let (min_dist, max_dist) = deformation_limits(problem, idx, *neighbour_idx);
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
    result
}


fn get_hole_mask(problem: &Problem) -> Array2<i16> {
    let (_, pt_max) = bounding_box(&problem.hole).unwrap();
    let xdim = pt_max.x + 1;
    let ydim = pt_max.y + 1;
    let mut result = Array2::ones((xdim as usize, ydim as usize));
    for x in 0..xdim {
        for y in 0..ydim {
            if pt_in_poly(Pt::new(x, y), &problem.hole) {
                result[[x as usize, y as usize]] = 0;
            }
        }
    }
    result
}

fn expand(problem: &Problem, vs: &mut Vec<Pt>, selected_idxs: &[usize], hole_mask: &Array2<i16>) {
    let mut cur_dislikes = get_dislikes(problem, vs);
    let mut prev_dislikes = cur_dislikes;
    loop {
        //dbg!(cur_dislikes);
        for idx in selected_idxs.iter() {
            //dbg!(idx);
            let positions = valid_positions(problem, vs, *idx, hole_mask);
            for pt in positions {
                let cur = vs[*idx];
                vs[*idx] = pt;
                let dislikes = get_dislikes(problem, vs);
                if dislikes < cur_dislikes {
                    cur_dislikes = dislikes;
                } else {
                    vs[*idx] = cur;
                }
            }
        }
        if cur_dislikes == prev_dislikes {
            break
        } else {
            prev_dislikes = cur_dislikes;
        }
    }
}

fn shake(problem: &Problem, vs: &mut Vec<Pt>, selected_idxs: &[usize], rng:  &mut dyn rand::RngCore, hole_mask: &Array2<i16>) -> i64 {
    let cur_dislikes = get_dislikes(problem, vs);
    for _ in 0..1 {
        for idx in selected_idxs.iter() {
            //dbg!(idx);
            let perturbations = valid_positions(problem, vs, *idx, hole_mask);
            let mut non_worsening_perturbations = vec![];
            let cur = vs[*idx];
            for pt in perturbations {
                vs[*idx] = pt;
                let dislikes = get_dislikes(problem, vs);
                if dislikes <= cur_dislikes {
                    non_worsening_perturbations.push(pt);
                }
            }
            // Actualy this shouldn't be empty because current position is in it.
            if non_worsening_perturbations.is_empty() {
                vs[*idx] = cur;
            } else {
                vs[*idx] = *non_worsening_perturbations.choose(rng).unwrap();
            }

        }
    }
    cur_dislikes
}

pub fn greedy_shake(r: &ShakeRequest) -> Vec<Pt> {
    dbg!(r.problem.figure.vertices.len(), r.problem.hole.len());
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
    
    let hole_mask = get_hole_mask(&r.problem);

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
        dbg!(i);
        expand(&r.problem, &mut cur_vs, &selected_idxs, &hole_mask);
        //dbg!("Shake");
        let cur_dislikes = shake(&r.problem, &mut cur_vs, &selected_idxs, &mut rng, &hole_mask);
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
