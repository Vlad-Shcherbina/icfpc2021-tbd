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

fn step(problem: &Problem, vs: &mut Vec<Pt>, selected_idxs: &[usize], rng:  &mut dyn rand::RngCore, hole_mask: &Array2<i16>, threshold: i64) {
    let cur_dislikes = get_dislikes(problem, vs);
    let mut selected_idxs_shuffled = selected_idxs.to_vec().clone();
    selected_idxs_shuffled.shuffle(rng);

    for idx in selected_idxs_shuffled.iter() {
        //dbg!(idx);
        let mut acceptable_perturbations = vec![];
        let cur = vs[*idx];
        for pt in valid_positions(problem, vs, *idx, hole_mask) {
            if pt == vs[*idx] {
                continue;
            }
            vs[*idx] = pt;
            let dislikes = get_dislikes(problem, vs);
            if dislikes - cur_dislikes <= threshold {
                acceptable_perturbations.push(pt);
            }
        }
        if acceptable_perturbations.is_empty() {
            //dbg!("empty");
            vs[*idx] = cur;
            // continue;
        } else {
            //dbg!("non-empty");
            vs[*idx] = *acceptable_perturbations.choose(rng).unwrap();
            return;
        }

    }
}

fn threshold(i: i64, param: i64) -> i64 {
    //dbg!(i, param);
    let nz = 10000;
    if i < nz {
        return (0.05 * param as f64 * (1.0 - (i as f64 / nz as f64))) as i64;
    } else {
        return 0;
    }
}

pub fn threshold_shake(r: &ShakeRequest) -> Vec<Pt> {
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
        dbg!("invalid pose passed to threshold shake");
        return cur_vs;
    }
    let mut dislikes = pr.dislikes;
    let convergence_cutoff = r.param*100;
    let mut j = 0;
    for i in 0.. {
        dbg!(i);
        let threshold = threshold(i, pr.dislikes);
        step(&r.problem, &mut cur_vs, &selected_idxs, &mut rng, &hole_mask, threshold);

        let cur_dislikes = get_dislikes(&r.problem, &cur_vs);
        //if threshold > 0 { dbg!(threshold); }
        dbg!(cur_dislikes);
        if cur_dislikes != dislikes {
            dislikes = cur_dislikes;
            j = 0;
        } else {
            j += 1;
            if j > convergence_cutoff {
                break;
            }
        }
    }

    cur_vs
}
