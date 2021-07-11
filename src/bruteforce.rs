use crate::shake::ShakeRequest;
use crate::prelude::{Problem, Pt};
use crate::graph::bfs;
use crate::mango::available_positions;
use rand::prelude::SliceRandom;
use crate::checker::{get_dislikes, Checker};
use std::collections::HashSet;

fn go(checker: &mut Checker,
      order: &Vec<usize>,
      offset: usize,
      places: &mut Vec<Option<Pt>>) -> Option<Vec<Pt>> {

    if offset < order.len() {
        let current_id = order[offset];

        let mut vertices: Vec<_> = places.iter().cloned().filter_map(|pt| pt).collect();
        let mut available_positions = available_positions(checker, places, current_id);
        available_positions.sort_by_key(|pt| {
            vertices.push(*pt);
            let dislikes = get_dislikes(&checker.problem, &vertices);
            vertices.pop();
            dislikes
        });

        // eprintln!("available_positions: {:?}", available_positions);
        for pt in available_positions.drain(0..) {
            places[current_id] = Some(pt);
            let result = go(checker, order, offset + 1, places);
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

fn brutforce_with(checker: &mut Checker, v_id: usize, pt: Pt) -> Option<Vec<Pt>> {
    let mut order = bfs(&checker.problem.figure.edges, v_id).iter().cloned().skip(1).collect();
    let mut places: Vec<Option<Pt>> = vec![None; checker.problem.figure.vertices.len()];
    places[v_id] = Some(pt);
    go(checker, &order, 0, &mut places)
}

pub fn brutforce(r: &ShakeRequest) -> Vec<Pt> {
    let rng = &mut rand::thread_rng();
    let mut v_ids: Vec<_> = (0..r.vertices.len()).into_iter().collect();
    v_ids.shuffle(rng);
    let mut h_pts = r.problem.hole.clone();
    h_pts.shuffle(rng);
    let mut checker = Checker::new(&r.problem, &vec![]);
    for v_id in v_ids {
        for (h_id, pt) in h_pts.iter().enumerate() {
            eprintln!("Trying: v_id {:?} h_id {:?}", v_id, h_id);
            if let Some(result) = brutforce_with(&mut checker, v_id, *pt) {
                eprintln!("Success!");
                return result;
            }
        }
    }
    eprintln!("No luck so far!");
    r.vertices.clone()
}