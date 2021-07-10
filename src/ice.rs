#![allow(unused_imports)]

use rand::Rng;
use crate::checker::length_range;
use crate::geom::segment_in_poly;
use crate::prelude::*;
use crate::shake::ShakeRequest;

pub fn ice_shake(r: &ShakeRequest) -> Vec<Pt> {
    let mut _pose = Pose {
        bonuses: vec![],
        vertices: r.vertices.clone(),
    };

    let mut selected = r.selected.clone();
    if selected.iter().all(|&s| !s) {
        selected = vec![true; selected.len()];
    }

    let selected_idx: Vec<usize> = selected.iter().enumerate()
        .filter_map(|(i, &sel)| if sel { Some(i) } else { None })
        .collect();

    dbg!(selected_idx);

    for _ in 1..10000 {

    }
    todo!()
}

crate::entry_point!("ice_demo", ice_demo);
fn ice_demo() {
    let p = load_problem(std::env::args().nth(2).unwrap());

    let r = ShakeRequest {
        vertices: p.figure.vertices.clone(),
        selected: vec![true; p.figure.vertices.len()],
        method: "ice".to_owned(),
        param: 42,
        problem: p,
    };
    ice_shake(&r);
}
