#![allow(unused_imports)]

use crate::shake::ShakeRequest;
use crate::prelude::{Pt, Problem};
use crate::graph::neighbours;
use std::cmp::max;
use crate::geom::{segment_in_poly, bounding_box};
use rand::prelude::SliceRandom;
use crate::checker::Checker;

struct Borders {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
}

fn borders(hole: &[Pt]) -> Borders {
    let (pt_min, pt_max) = bounding_box(hole).unwrap();
    Borders {
        min_x: pt_min.x,
        max_x: pt_max.x,
        min_y: pt_min.y,
        max_y: pt_max.y
    }
}

fn orig_distance(problem: &Problem, v1_id: usize, v2_id: usize) -> i64 {
    problem.figure.vertices[v1_id].dist2(problem.figure.vertices[v2_id])
}

fn deformation_limits(problem: &Problem, v1_id: usize, v2_id: usize) -> (i64, i64) {
    let orig_d2 = orig_distance(problem, v1_id, v2_id);
    crate::checker::length_range(orig_d2, problem.epsilon)
}

pub fn available_positions(checker: &mut Checker, vertices: &[Option<Pt>], v_id: usize) -> Vec<Pt> {
    let borders = borders(&checker.problem.hole);
    let mut available_positions = vec![];
    for x in borders.min_x..=borders.max_x {
        'l: for y in borders.min_y..=borders.max_y {
            let assumed_pos = Pt { x, y };

            if Some(assumed_pos) == vertices[v_id] {
                continue 'l;
            }

            for n_id in checker.neighbours(v_id).clone() {
                if let Some(n) = vertices[n_id] {
                    let (min_dist, max_dist) = deformation_limits(&checker.problem, v_id, n_id);
                    let new_dist = assumed_pos.dist2(n);
                    if !(min_dist <= new_dist && new_dist <= max_dist) {
                        // eprintln!("Drop {:?} by distance", assumed_pos);
                        continue 'l;
                    }
                    if !segment_in_poly((n, assumed_pos), &checker.problem.hole) {
                        // eprintln!("Drop {:?} by intersection", assumed_pos);
                        continue 'l;
                    }
                }
            }
            available_positions.push(assumed_pos);
        }
    }
    available_positions
}

pub fn mango_shake(r: &ShakeRequest) -> Vec<Pt> {
    let rng = &mut rand::thread_rng();
    let mut result: Vec<_> = r.vertices.iter().map(|pt| Some(*pt)).collect();
    let mut selected_idxs: Vec<_> = r.selected.iter().enumerate()
        .filter(|(_, b)| **b)
        .map(|(idx, _)| idx)
        .collect();

    let mut success = false;
    let mut iteration_count = 0;
    let mut checker = Checker::new(&r.problem, &vec![]);
    loop {
        selected_idxs.shuffle(rng);
        for v_id in &selected_idxs {
            let available_positions = available_positions(&mut checker, &result, *v_id);
            if !available_positions.is_empty() {
                result[*v_id] = Some(*available_positions.choose(rng).unwrap());
                success = true;
            }
        }
        iteration_count += 1;
        if success || iteration_count > 10 {
            return result.iter().map(|pt| pt.unwrap()).collect();
        }
    }
}