use crate::shake::ShakeRequest;
use crate::prelude::{Pt, Problem};
use crate::graph::neighbours;
use std::cmp::max;
use crate::geom::segment_in_poly;
use rand::prelude::SliceRandom;

struct Borders {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
}

fn borders(hole: &Vec<Pt>) -> Borders {
    let min_x = hole.iter().map(|pt| pt.x).min().unwrap();
    let max_x = hole.iter().map(|pt| pt.x).max().unwrap();
    let min_y = hole.iter().map(|pt| pt.x).min().unwrap();
    let max_y = hole.iter().map(|pt| pt.x).max().unwrap();
    return Borders {
        min_x,
        max_x,
        min_y,
        max_y
    };
}

fn orig_distance(problem: &Problem, v1_id: usize, v2_id: usize) -> i64 {
    problem.figure.vertices[v1_id].dist2(problem.figure.vertices[v2_id])
}

fn deformation_limits(problem: &Problem, v1_id: usize, v2_id: usize) -> (i64, i64) {
    let orig_d2 = orig_distance(problem, v1_id, v2_id);
    return crate::checker::length_range(orig_d2, problem.epsilon);
}

pub fn mango_shake(r: &ShakeRequest) -> Vec<Pt> {
    eprintln!("Start mango shaking");
    let rng = &mut rand::thread_rng();
    let mut vs = r.vertices.clone();
    assert_eq!(vs.len(), r.selected.len());
    for v_id in 0..vs.len() {
        if r.selected[v_id] {
            let neighbours: Vec<_> = neighbours(&r.problem.figure.edges, v_id).collect();
            let borders = borders(&r.problem.hole);
            let mut availiable_positions = vec![];
            for x in borders.min_x..=borders.max_x {
                'l: for y in borders.min_y..=borders.max_y {
                    let assumed_pos = Pt { x, y };

                    if assumed_pos == r.vertices[v_id] {
                        continue 'l;
                    }

                    for n_id in &neighbours {
                        let n = vs[*n_id];
                        let (min_dist, max_dist) = deformation_limits(&r.problem, v_id, *n_id);
                        let new_dist = assumed_pos.dist2(n);
                        if !(min_dist <= new_dist && new_dist <= max_dist) {
                            // eprintln!("Drop {:?} by distance", assumed_pos);
                            continue 'l;
                        }
                        if !segment_in_poly((n, assumed_pos), &r.problem.hole) {
                            // eprintln!("Drop {:?} by intersection", assumed_pos);
                            continue 'l;
                        }
                    }
                    availiable_positions.push(assumed_pos);
                }
            }

            if !availiable_positions.is_empty() {
                vs[v_id] = *availiable_positions.choose(rng).unwrap();
            }
        }
    }
    vs
}