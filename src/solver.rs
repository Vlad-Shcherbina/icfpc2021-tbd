use core::time::Duration;

use crate::prelude::*;
use crate::prelude::Problem;
use crate::random::RandomSolver;
use crate::util::{load_problem, store_solution};

pub enum SolverError {
    SolverOutOfTime(Duration),
    Other(String),
}

pub trait Solver {
    fn solve(&mut self, problem: &Problem, duration_limit: Option<Duration>) -> Result<Pose, SolverError>;
}

crate::entry_point!("solver", solver_main);
fn solver_main() {
    let problem_no = match std::env::args().nth(2) {
        Some(p) => p,
        None => {
            eprintln!("Usage:");
            eprintln!("    cargo run solver 11");
            eprintln!("    Env vars:");
            eprintln!("    SOLVER: one of: random, ..");
            eprintln!("    DURATION_LIMIT_SECONDS: time limit for solver");
            std::process::exit(1);
        }
    };

    let mut solver = match std::env::var("SOLVER") {
        Ok(solver_name) => match solver_name.as_str() {
            "random" => Box::new(RandomSolver {}),
            _ => Box::new(RandomSolver {}),
        }
        _ => Box::new(RandomSolver {}),
    };

    let duration_per_task = std::env::var("DURATION_LIMIT_SECONDS").ok().and_then(|duration_limit| {
        if let Ok(duration_limit) = duration_limit.parse::<u64>() {
            Some(Duration::from_secs(duration_limit))
        } else {
            None
        }
    });

    let problem: Problem = load_problem(&problem_no);
    match solver.solve(&problem, duration_per_task) {
        Ok(pose) => store_solution(&problem_no, &pose),
        Err(err) => eprintln!("Error: $err"),
    }
}