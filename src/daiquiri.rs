use rand::Rng;
use rand::prelude::ThreadRng;
use crate::checker::length_range;
// use crate::geom::segment_in_poly;
use crate::prelude::*;
use crate::shake::ShakeRequest;

pub fn daikuiri_shake(r: &ShakeRequest) -> Vec<Pt> {
    let mut vs = r.vertices.clone();
    let mut rng = rand::thread_rng();
    let ranges: Vec<(i64, i64)> = r.problem.figure.edges.iter().map(
        |e| {
            let &(start, end) = e;
            length_range(r.problem.figure.vertices[start].dist2(
                         r.problem.figure.vertices[end]),
                         r.problem.epsilon)
        }).collect();

    for _ in 0..100 {
        let mut not_visited: Vec<usize> = vec![];
        for i in 0..r.selected.len() {
            if r.selected[i] { not_visited.push(i); }
        }
        rand_permutation(&mut not_visited, &mut rng);
        for i in not_visited {
            shake_one(&mut vs, i, r, &ranges, &mut rng);
        }

        // let mut continuous = false;

        // while !not_visited.is_empty() {
        //     let mut current = 0;
        //     if !continuous {
        //         current = not_visited.pop().unwrap();
        //     }
        //     else {
        //         continuous = false;
        //         for idx in 0..not_visited.len() {
        //             if !is_adjacent(current, not_visited[idx], r) {
        //                 continue;
        //             }
        //             current = not_visited.remove(idx);
        //             continuous = true;
        //             break;
        //         }
        //     }
        //     if !continuous { continue; }
        //     shake_one(&mut vs, current, r, &ranges, &mut rng);
        // }
    }
    vs
}

fn is_adjacent(i: usize, j: usize, r: &ShakeRequest) -> bool {
    if i == j { return false; }
    for &(start, end) in &r.problem.figure.edges {
        if (start == i || end == i) &&  (start == j || end == j) {
            return true;
        }
    }
    false
}

fn rand_permutation(a: &mut [usize], rng: &mut ThreadRng) {
    for i in 0..a.len() {
        a.swap(i, rng.gen_range(0..i+1));
    }
}

fn shake_one(vs: &mut [Pt], i: usize, r: &ShakeRequest, ranges: &[(i64, i64)], 
             rng: &mut ThreadRng) {
    let mut adj_edges = vec![];
    let edges = &r.problem.figure.edges;
    for e in 0..(*edges).len() {
        let &(start, end) = &(*edges)[e];
        if start == i || end == i { adj_edges.push(e); }
    }
    for _ in 0..100 {
        for &e in &adj_edges {
            let j = if edges[e].0 == i { edges[e].1 } else { edges[e].0 };
            let d = vs[i].dist2(vs[j]);
            let delta = 
            if ranges[e].0 > d {
                1
            }
            else if ranges[e].1 < d {
                -1
            }
            else { continue; };
            if rng.gen_bool(0.5) {
                vs[i].x += if vs[j].x > vs[i].x { -delta } else { delta };
            }
            else {
                vs[i].y += if vs[j].y > vs[i].y { -delta } else { delta };
            }
        }
    }
}
