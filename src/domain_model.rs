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

impl Problem {
    pub fn weight(&self) -> f64 {
        let w = self.figure.vertices.len() * self.figure.edges.len() * self.hole.len();
        (w as f64 / 6.0).log2() * 1000.0
    }

    pub fn normalized_score(&self, dislikes: i64, best_dislikes: i32) -> i64 {
        let t = (best_dislikes as f64 + 1.0) / (dislikes as f64 + 1.0);
        (t * self.weight()).ceil() as i64
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "bonustype")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum BonusName {
    GLOBALIST,
    BREAK_A_LEG,
    WALLHACK,
    SUPERFLEX,
}

impl BonusName {
    pub fn short_name(&self) -> String {
        format!("{:?}", self)[..1].to_owned()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct ProblemBonus {
    pub bonus: BonusName,
    pub problem: i32,
    pub position: Pt,
}

#[derive(serde::Serialize)]
#[derive(Debug, Clone)]
pub struct ProblemTgtBonus {
    pub bonus: BonusName,
    pub from_problem: i32,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Clone)]
pub struct PoseBonus {
    pub bonus: BonusName,
    pub problem: i32,
    pub edge: Option<(usize, usize)>,
}

impl std::fmt::Debug for PoseBonus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.problem, self.bonus.short_name())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct Pose {
    #[serde(default)] pub bonuses: Vec<PoseBonus>,
    pub vertices: Vec<Pt>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct UnlockedBonus {
    pub name: BonusName,
    pub problem_id: i32,
}

impl std::fmt::Debug for UnlockedBonus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.name.short_name(), self.problem_id)
    }
}

#[test]
fn test_problem_parsing() {
    let path = project_path("data/problems/1.problem");
    let data = std::fs::read(path).unwrap();
    let problem: Problem = serde_json::from_slice(&data).unwrap();
    dbg!(problem);
}
