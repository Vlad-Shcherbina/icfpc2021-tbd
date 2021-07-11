use crate::shake::ShakeRequest;
use crate::prelude::{Problem, Pt};
use std::collections::{HashMap, HashSet};
use crate::graph::bfs;
use crate::mango::available_positions;

fn selected_idxs(selected: &Vec<bool>) -> Vec<usize> {
    selected.iter().enumerate()
        .filter(|(_, b)| **b)
        .map(|(idx, _)| idx)
        .collect()
}

fn go(problem: &Problem, free_nodes: &mut Vec<usize>, places: &mut Vec<Option<Pt>>) -> Option<Vec<Pt>> {
    // in bfs order
    // place vertex in any availiable position according already placed
    // until all vertex will be placed
    // check

    while !free_nodes.is_empty() {
        let current_id = free_nodes.pop().unwrap();
        let mut availiable_positions = available_positions(problem, places, current_id);
        // eprintln!("availiable_positions: {:?}", availiable_positions);
        for pt in availiable_positions.drain(0..) {
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
        return Some(places.iter().map(|pt| pt.unwrap()).collect());
    } else {
        return None;
    }
}

fn brutforce_with(problem: &Problem, v_id: usize, pt: Pt) -> Option<Vec<Pt>> {
    let mut order = bfs(&problem.figure.edges, v_id);
    order.reverse();
    order.pop();
    let mut places: Vec<Option<Pt>> = vec![None; problem.figure.vertices.len()];
    places[v_id] = Some(pt);
    go(&problem, &mut order, &mut places)
}

pub fn brutforce(r: &ShakeRequest) -> Vec<Pt> {
    for v_id in 0..r.vertices.len() {
        for (h_id, pt) in r.problem.hole.iter().enumerate() {
            eprintln!("Trying: v_id {:?} h_id {:?}", v_id, h_id);
            if let Some(result) = brutforce_with(&r.problem, v_id, *pt) {
                return result;
            }
        }
    }
    return r.vertices.clone();

}