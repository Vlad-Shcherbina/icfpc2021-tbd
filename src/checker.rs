use std::collections::HashMap;
use crate::prelude::*;
use crate::geom::{bounding_box, pt_in_poly, segment_in_poly};

#[derive(serde::Deserialize)]
pub struct CheckPoseRequest {
    pub problem: Problem,
    pub pose: Pose,
}

#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct CheckPoseResponse {
    pub edge_statuses: Vec<EdgeStatus>,
    pub dislikes: i64,
    pub valid: bool,
    pub unlocked: Vec<bool>,
}

#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct EdgeStatus {
    pub fits_in_hole: bool,
    pub actual_length: i64,
    pub min_length: i64,
    pub max_length: i64,
}

impl EdgeStatus {
    fn is_valid(&self) -> bool {
        self.fits_in_hole &&
        self.min_length <= self.actual_length &&
        self.actual_length <= self.max_length
    }
}

// inclusive
pub fn length_range(d: i64, eps: i64) -> (i64, i64) {
    let min_length = (d * (EPS_BASE - eps) + EPS_BASE - 1) / EPS_BASE;
    let max_length = d * (EPS_BASE + eps) / EPS_BASE;
    (min_length, max_length)
}

// Precomputed data to quickly check pose constraints.
pub struct Checker {
    pub problem: Problem,
    pub edge_ranges: Vec<(i64, i64)>,
    pub edges: Vec<(usize, usize)>,
    pub inside: Vec<Pt>,
    pub edge_cache: HashMap<(Pt, Pt), bool>,
}

impl Checker {
    pub fn new(p: &Problem, used_bonuses: &[PoseBonus]) -> Checker {
        assert!(used_bonuses.is_empty(), "TODO");
        let edge_ranges = p.figure.edges.iter()
            .map(|&(start, end)| {
                let d = p.figure.vertices[start].dist2(p.figure.vertices[end]);
                length_range(d, p.epsilon)
            }).collect();

        let (pt_min, pt_max) = bounding_box(&p.hole).unwrap();
        let mut inside = vec![];
        for x in pt_min.x..=pt_max.x {
            for y in pt_min.y..=pt_max.y {
                let pt = Pt::new(x, y);
                if pt_in_poly(pt, &p.hole) {
                    inside.push(pt);
                }
            }
        }

        Checker {
            problem: p.clone(),
            edges: p.figure.edges.clone(),
            edge_ranges,
            inside,
            edge_cache: HashMap::new(),
        }
    }

    pub fn edge_in_hole(&mut self, mut pt1: Pt, mut pt2: Pt) -> bool {
        if (pt1.x, pt1.y) > (pt2.x, pt1.y) {
            std::mem::swap(&mut pt1, &mut pt2);
        }
        let hole = &self.problem.hole;
        *self.edge_cache.entry((pt1, pt2)).or_insert_with(|| {
            segment_in_poly((pt1, pt2), hole)
        })
    }
}

pub fn get_dislikes(problem: &Problem, vertices: &[Pt]) -> i64 {
    let mut dislikes = 0;
    for &h in &problem.hole {
        dislikes += vertices.iter().map(|v| v.dist2(h)).min().unwrap();
    }
    dislikes
}

pub fn check_unlocked(problem: &Problem, vertices: &[Pt]) -> Vec<bool> {
    let mut unlocked: Vec<bool> = vec![false; problem.bonuses.len()];
    for v in vertices {
        for i in 0..unlocked.len() {
            if *v == problem.bonuses[i].position {
                unlocked[i] = true;
            }
        }
    }
    unlocked
}

pub fn check_pose(problem: &Problem, pose: &Pose) -> CheckPoseResponse {
    let mut checker = Checker::new(problem, &pose.bonuses);

    let vertices = &pose.vertices;
    assert_eq!(problem.figure.vertices.len(), vertices.len());

    let mut edge_statuses = vec![];
    let mut valid = true;
    let mut unlocked = check_unlocked(problem, vertices);
    for i in 0..problem.figure.edges.len() {
        let pt1 = vertices[problem.figure.edges[i].0];
        let pt2 = vertices[problem.figure.edges[i].1];

        let fits_in_hole = checker.edge_in_hole(pt1, pt2);

        let (min_length, max_length) = checker.edge_ranges[i];

        let es = EdgeStatus {
            fits_in_hole,
            actual_length: pt1.dist2(pt2),
            min_length,
            max_length,
        };
        valid = valid && es.is_valid();
        edge_statuses.push(es);
    }

    for _b in &problem.bonuses {
        unlocked.push(false);
    }

    let dislikes = get_dislikes(problem, vertices);

    CheckPoseResponse {
        edge_statuses,
        dislikes,
        valid,
        unlocked,
    }
}

#[test]
fn test_check_pose() {
    let p = crate::util::load_problem(1);
    let pose = Pose {
        vertices: p.figure.vertices.clone(),
        bonuses: vec![],
    };
    dbg!(check_pose(&p, &pose));
}
