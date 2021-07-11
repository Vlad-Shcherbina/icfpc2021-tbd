use rand::Rng;
use rand::prelude::ThreadRng;
use crate::checker::length_range;
use crate::geom::pt_in_poly;
use crate::prelude::*;
use crate::shake::ShakeRequest;

const COEFF: i64 = 1;

pub fn daikuiri_shake(r: &ShakeRequest, mojito: bool) -> Vec<Pt> {
    let mut vs = r.vertices.clone();
    let mut rng = rand::thread_rng();
    let in_hole: Vec<bool> = vs.iter().map(|v| pt_in_poly(*v, &r.problem.hole)).collect();
    let ranges: Vec<(i64, i64)> = r.problem.figure.edges.iter().map(
        |e| {
            let &(start, end) = e;
            length_range(r.problem.figure.vertices[start].dist2(
                         r.problem.figure.vertices[end]),
                         r.problem.epsilon)
        }).collect();

    for _ in 0..r.param * r.param * COEFF * (r.selected.len() as i64) {
        let mut not_visited: Vec<usize> = vec![];
        for i in 0..r.selected.len() {
            if r.selected[i] { not_visited.push(i); }
        }
        rand_permutation(&mut not_visited, &mut rng);
        for i in not_visited {
            shake_one(&mut vs, i, r, &ranges, &mut rng, mojito && in_hole[i]);
        }
    }
    vs
}

// fn is_adjacent(i: usize, j: usize, r: &ShakeRequest) -> bool {
//     if i == j { return false; }
//     for &(start, end) in &r.problem.figure.edges {
//         if (start == i || end == i) &&  (start == j || end == j) {
//             return true;
//         }
//     }
//     false
// }

fn rand_permutation(a: &mut [usize], rng: &mut ThreadRng) {
    for i in 0..a.len() {
        a.swap(i, rng.gen_range(0..i+1));
    }
}

fn shake_one(vs: &mut [Pt], i: usize, r: &ShakeRequest, ranges: &[(i64, i64)], 
             rng: &mut ThreadRng, keep_in_hole: bool) {
    let mut adj_edges = vec![];
    let edges = &r.problem.figure.edges;
    for e in 0..(*edges).len() {
        let &(start, end) = &(*edges)[e];
        if start == i || end == i { adj_edges.push(e); }
    }
    for _ in 0..r.param * r.param * COEFF {
        for &e in &adj_edges {
            let j = if edges[e].0 == i { edges[e].1 } else { edges[e].0 };
            let d = vs[i].dist2(vs[j]);
            let extend = if ranges[e].0 > d { 1 } else if ranges[e].1 < d { -1 }
                         else { continue; };
            
            let [mut dx, mut dy] = [0, 0];
            let choose_x = rng.gen_bool(0.5);
            let choose_both = rng.gen_bool(0.1);
            if choose_x || choose_both {
                dx = if vs[j].x > vs[i].x { -extend } else { extend };
            }
            if !choose_x || choose_both {
                dy = if vs[j].y > vs[i].y { -extend } else { extend };
            }
            vs[i].x += dx;
            vs[i].y += dy;
            if keep_in_hole && !pt_in_poly(vs[i], &r.problem.hole) {
                vs[i].x -= dx;
                vs[i].y -= dy;    
            }
        }
    }
}
