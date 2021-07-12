#![allow(unused_imports)]

use rand::Rng;
use crate::checker::{length_range, check_pose, get_dislikes};
use crate::geom::*;
use crate::prelude::*;
use crate::graph::*;
use crate::shake::ShakeRequest;
use rand::prelude::SliceRandom;
use ndarray::Array2;

// Quick & dirty code reuse.
use crate::threshold::{orig_distance, deformation_limits, HoleChecker, valid_positions};

fn expand(problem: &Problem, vs: &mut Vec<Pt>, selected_idxs: &[usize], hole_checker: &HoleChecker) {
    let mut cur_dislikes = get_dislikes(problem, vs);
    let mut prev_dislikes = cur_dislikes;
    loop {
        //dbg!(cur_dislikes);
        for idx in selected_idxs.iter() {
            //dbg!(idx);
            let positions = valid_positions(problem, vs, *idx, hole_checker);
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

fn shake(problem: &Problem, vs: &mut Vec<Pt>, selected_idxs: &[usize], rng:  &mut dyn rand::RngCore, hole_checker: &HoleChecker) -> i64 {
    let cur_dislikes = get_dislikes(problem, vs);
    for _ in 0..1 {
        for idx in selected_idxs.iter() {
            //dbg!(idx);
            let perturbations = valid_positions(problem, vs, *idx, hole_checker);
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
    //assert!(r.problem.bonuses.is_empty());
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
    
    let hole_checker = HoleChecker::new(&r.problem);

    let mut cur_vs = r.vertices.clone();

    let pr = check_pose(&r.problem, &Pose { vertices: cur_vs.clone(), bonuses: vec![] });
    if !pr.valid {
        dbg!("invalid pose passed to greedy shake");
        return cur_vs;
    }
    let mut dislikes = pr.dislikes;
    let convergence_cutoff = r.param*50;
    let mut i = 0;
    loop {
        if i % 10 == 0 {
            dbg!(i);
        }
        expand(&r.problem, &mut cur_vs, &selected_idxs, &hole_checker);
        //dbg!("Shake");
        let cur_dislikes = shake(&r.problem, &mut cur_vs, &selected_idxs, &mut rng, &hole_checker);
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
