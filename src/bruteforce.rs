use crate::shake::ShakeRequest;
use crate::prelude::{Problem, Pt};
use crate::graph::bfs;
use crate::mango::available_positions;
use rand::prelude::SliceRandom;
use crate::checker::get_dislikes;


fn go(problem: &Problem,
      free_nodes: &mut Vec<usize>,
      places: &mut Vec<Option<Pt>>) -> Option<Vec<Pt>> {
    // in bfs order
    // place vertex in any availiable position according already placed
    // until all vertex will be placed
    // check

    while !free_nodes.is_empty() {
        let current_id = free_nodes.pop().unwrap();

        let mut vertices: Vec<_> = places.iter().filter_map(|pt| *pt).collect();
        let mut available_positions = available_positions(problem, places, current_id);
        available_positions.sort_by_cached_key(|pt| {
            vertices.push(*pt);
            let dislikes = get_dislikes(problem, &vertices);
            vertices.pop();
            dislikes
        });

        // eprintln!("availiable_positions: {:?}", availiable_positions);
        for pt in available_positions.drain(0..) {
            places[current_id] = Some(pt);
            let result = go(problem, free_nodes, places);
            if result.is_some() {
                return result;
            }
            places[current_id] = None;
        }
    }

    // eprintln!("places: {:?}", places);

    if places.iter().all(|pt| pt.is_some()) {
        Some(places.iter().map(|pt| pt.unwrap()).collect())
    } else {
        None
    }
}

fn brutforce_with(problem: &Problem, v_id: usize, pt: Pt) -> Option<Vec<Pt>> {
    let mut order = bfs(&problem.figure.edges, v_id);
    order.reverse();
    order.pop();
    let mut places: Vec<Option<Pt>> = vec![None; problem.figure.vertices.len()];
    places[v_id] = Some(pt);
    go(problem, &mut order, &mut places)
}

pub fn brutforce(r: &ShakeRequest) -> Vec<Pt> {
    let rng = &mut rand::thread_rng();
    let mut v_ids: Vec<_> = (0..r.vertices.len()).into_iter().collect();
    v_ids.shuffle(rng);
    let mut h_ids = r.problem.hole.clone();
    h_ids.shuffle(rng);
    for v_id in v_ids {
        for (h_id, pt) in h_ids.iter().enumerate() {
            eprintln!("Trying: v_id {:?} h_id {:?}", v_id, h_id);
            if let Some(result) = brutforce_with(&r.problem, v_id, *pt) {
                return result;
            }
        }
    }
    r.vertices.clone()
}