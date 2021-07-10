use rand::prelude::*;
use crate::prelude::*;
use crate::geom::segment_in_poly;
use crate::util::{load_problem, store_solution};
use crate::solver::{Solver, SolverError};
use core::time::Duration;
use std::time::Instant;

pub struct RandomSolver;

impl Solver for RandomSolver {
    fn solve(&mut self, problem: &Problem, duration_limit: Option<Duration>) -> Result<Pose, SolverError> {
        let start_time = Instant::now();
        dbg!(duration_limit);
        let mut rng = StdRng::seed_from_u64(42);

        dbg!(problem.figure.vertices.len());
        let vertices = &problem.figure.vertices;
        let x1 = vertices.iter().map(|pt| pt.x).min().unwrap();
        let y1 = vertices.iter().map(|pt| pt.y).min().unwrap();
        let x2 = vertices.iter().map(|pt| pt.x).max().unwrap();
        let y2 = vertices.iter().map(|pt| pt.y).max().unwrap();

        dbg!((x1, y1, x2, y2));
        dbg!(problem.epsilon);
        let mut cnt = 0;
        let solution;
        loop {
            if let Some(duration_limit) = duration_limit {
                if start_time.elapsed() > duration_limit {
                    return Err(SolverError::SolverOutOfTime(start_time.elapsed()));
                }
            }
            cnt += 1;
            if cnt % 10000000 == 0 {
                dbg!(cnt);
            }

            let pose: Vec<Pt> = vertices.iter()
                .map(|_| Pt::new(rng.gen_range(x1..=x2), rng.gen_range(y1..=y2)))
                .collect();

            let mut good = true;
            for &(start, end) in &problem.figure.edges {
                let orig_d2 = vertices[start].dist2(vertices[end]);
                let new_d2 = pose[start].dist2(pose[end]);

                let (min_d2, max_d2) = crate::checker::length_range(orig_d2, problem.epsilon);

                if new_d2 < min_d2 || new_d2 > max_d2 {
                    good = false;
                    break;
                }

                if !segment_in_poly((pose[start], pose[end]), &problem.hole) {
                    good = false;
                    break;
                }
            }
            if !good {
                continue;
            }

            dbg!(cnt);
            dbg!(&pose);
            solution = Some(Pose { vertices: pose, bonuses: vec![] });
            break;
        }
        Ok(solution.unwrap())
    }
}

crate::entry_point!("random_solver", random_solver);
fn random_solver() {
    let problem_no = match std::env::args().nth(2) {
        Some(p) => p,
        None => {
            eprintln!("Usage:");
            eprintln!("    cargo run random_solver 11");
            std::process::exit(1);
        }
    };

    let problem: Problem = load_problem(&problem_no);
    let mut solver = RandomSolver;
    let pose = solver.solve(&problem, None).ok().unwrap();
    store_solution(&problem_no, &pose);
}
