use crate::shake::ShakeRequest;
use crate::prelude::{Pt, Problem};
use crate::graph::neighbours;
use rand::prelude::SliceRandom;

fn orig_distance(problem: &Problem, v1_id: usize, v2_id: usize) -> i64 {
    problem.figure.vertices[v1_id].dist2(problem.figure.vertices[v2_id])
}

pub fn springs(r: &ShakeRequest) -> Vec<Pt> {
    let rng = &mut rand::thread_rng();
    let mut result = r.vertices.clone();
    let rigidity = 0.2;

    let mut selected_idxs: Vec<_> = r.selected.iter().enumerate()
        .filter(|(_, b)| **b)
        .map(|(idx, _)| idx)
        .collect();
    selected_idxs.shuffle(rng);

    for v_id in selected_idxs {
        let mut dx = 0.0;
        let mut dy = 0.0;
        for n_id in neighbours(&r.problem.figure.edges, v_id) {
            let v = r.vertices[v_id];
            let n = r.vertices[n_id];
            let orig_distance = (orig_distance(&r.problem, v_id, n_id) as f64).sqrt();
            let current_dist = (v.dist2(n) as f64).sqrt();
            dx += rigidity * ((n - v).x as f64 / current_dist * (current_dist - orig_distance));
            dy += rigidity * ((n - v).y as f64 / current_dist * (current_dist - orig_distance));
        }
        eprintln!("dx: {:?} dy: {:?}", dx, dy);
        result[v_id] = Pt {
            x: result[v_id].x + dx.round() as i64,
            y: result[v_id].y + dy.round() as i64
        }
    }
    result
}