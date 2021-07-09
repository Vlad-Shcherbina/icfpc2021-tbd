// Things that are defined in the problem statement.

use crate::prelude::*;

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct Figure {
    pub vertices: Vec<Pt>,
    pub edges: Vec<(usize, usize)>,
}

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct Problem {
    pub hole: Vec<Pt>,
    pub figure: Figure,
    pub epsilon: f64,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug)]
pub struct Pose {
    pub vertices: Vec<Pt>,
}

#[test]
fn test_problem_parsing() {
    let path = project_path("data/problems/1.problem");
    let data = std::fs::read(path).unwrap();
    let problem: Problem = serde_json::from_slice(&data).unwrap();
    dbg!(problem);
}
