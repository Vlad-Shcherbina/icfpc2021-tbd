use rand::Rng;
use crate::checker::length_range;
use crate::geom::segment_in_poly;
use crate::prelude::*;
use crate::shake::ShakeRequest;

pub fn banana_shake(r: &ShakeRequest) -> Vec<Pt> {

    let compute_score = |vs: &[Pt]| {
        let mut score: f64 = 0.0;
        for &(start, end) in &r.problem.figure.edges {
            if !r.selected[start] || !r.selected[end] {
                continue;
            }
            if !segment_in_poly((vs[start], vs[end]), &r.problem.hole) {
                score -= 1.0;
            }
            let orig_d = r.problem.figure.vertices[start].dist2(r.problem.figure.vertices[end]);
            let (min_d, max_d) = length_range(orig_d, r.problem.epsilon);
            let d = vs[start].dist2(vs[end]);
            if d < min_d {
                score -= (min_d - d) as f64 / orig_d as f64;
            }
            if d > max_d {
                score -= (d - max_d) as f64 / orig_d as f64;
            }
        }
        score
    };

    let mut best = r.vertices.clone();
    let mut best_score = compute_score(&r.vertices);

    let mut selected_idxs = vec![];
    for (i, &sel) in r.selected.iter().enumerate() {
        if sel {
            selected_idxs.push(i);
        }
    }

    let mut rng = rand::thread_rng();
    let mut cur = r.vertices.clone();
    let mut cur_score = compute_score(&cur);
    for _ in 0..1000 {
        let old = cur.clone();
        let old_score = cur_score;
        for _ in 0..r.param {
            let i = rng.gen_range(0..selected_idxs.len());
            let i = selected_idxs[i];
            cur[i].x += rng.gen_range(-1..=1);
            cur[i].y += rng.gen_range(-1..=1);
        }
        cur_score = compute_score(&cur);
        if cur_score >= old_score {
            if cur_score > best_score {
                best_score = cur_score;
                best = cur.clone();
            }
        } else {
            cur = old;
            cur_score = old_score;
        }
    }
    best
}
