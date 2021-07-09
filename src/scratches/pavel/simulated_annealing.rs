#![allow(unused_imports)]

use rand::prelude::*;
use crate::prelude::*;
use crate::geom::segment_in_poly;

crate::entry_point!("simulated_annealing", simulated_annealing_solver);
fn simulated_annealing_solver() {
    let _problem_id = match std::env::args().nth(2) {
        Some(p) => p,
        None => {
            eprintln!("Usage:");
            eprintln!("    cargo run random_solver 11");
            std::process::exit(1);
        }
    };


}