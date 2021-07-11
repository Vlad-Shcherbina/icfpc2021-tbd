// Things that are defined in the problem statement.

use crate::prelude::*;

pub const EPS_BASE: i64 = 1_000_000;

#[derive(serde::Deserialize)]
#[derive(Debug, Clone)]
pub struct Figure {
    pub vertices: Vec<Pt>,
    pub edges: Vec<(usize, usize)>,
}

#[derive(serde::Deserialize)]
#[derive(Debug, Clone)]
pub struct Problem {
    pub bonuses: Vec<ProblemBonus>,
    pub hole: Vec<Pt>,
    pub figure: Figure,
    pub epsilon: i64,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum BonusName {
    GLOBALIST,
    BREAK_A_LEG,
}

#[derive(serde::Deserialize)]
#[derive(Debug, Clone)]
pub struct ProblemBonus {
    pub bonus: BonusName,
    pub problem: u32,
    pub position: Pt,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct PoseBonus {
    pub bonus: BonusName,
    pub problem: u32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct Pose {
    pub bonuses: Vec<PoseBonus>,
    pub vertices: Vec<Pt>,
}

#[test]
fn test_problem_parsing() {
    let path = project_path("data/problems/1.problem");
    let data = std::fs::read(path).unwrap();
    let problem: Problem = serde_json::from_slice(&data).unwrap();
    dbg!(problem);
}
