use std::collections::HashMap;
use std::convert::TryInto;
use crate::domain_model::BonusName;
use crate::prelude::*;
use crate::geom::{bounding_box, pt_in_poly, segment_in_poly};
use crate::graph::neighbours;

#[derive(serde::Deserialize)]
pub struct CheckPoseRequest {
    pub problem: Problem,
    pub pose: Pose,
}

#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct CheckPoseResponse {
    pub edges: Vec<(usize, usize)>,
    pub edge_statuses: Vec<EdgeStatus>,
    pub dislikes: i64,
    pub valid: bool,
    pub unlocked: Vec<bool>,
    pub bonus_globalist_sum: Option<f32>
}

#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct EdgeStatus {
    pub fits_in_hole: bool,
    pub actual_length: i64,
    pub min_length: i64,
    pub max_length: i64,
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
    pub edge_cache: HashMap<[i16; 4], bool>,
    pub neighbours_cache: HashMap<usize, Vec<usize>>,
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
            neighbours_cache: HashMap::new(),
        }
    }

    pub fn edge_in_hole(&mut self, mut pt1: Pt, mut pt2: Pt) -> bool {
        if (pt1.x, pt1.y) > (pt2.x, pt1.y) {
            std::mem::swap(&mut pt1, &mut pt2);
        }
        let hole = &self.problem.hole;
        let key = [
            pt1.x.try_into().unwrap(),
            pt1.y.try_into().unwrap(),
            pt2.x.try_into().unwrap(),
            pt2.y.try_into().unwrap(),
        ];
        *self.edge_cache.entry(key).or_insert_with(|| {
            segment_in_poly((pt1, pt2), hole)
        })
    }

    pub fn neighbours(&mut self, v_id: usize) -> &Vec<usize> {
        let Checker { neighbours_cache, problem, .. } = self;
        neighbours_cache.entry(v_id).or_insert_with(|| {
            neighbours(&problem.figure.edges, v_id).collect()
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

#[allow(clippy::needless_range_loop)]
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

#[allow(clippy::needless_range_loop)]
pub fn check_edges_in_hole(problem: &Problem, pose: &Pose, 
        edge_statuses: &[EdgeStatus], checker: &Checker) -> bool {
    let mut wallhack: Option<usize> = None;
    for i in 0..edge_statuses.len() {
        if edge_statuses[i].fits_in_hole { continue; }
        if !pose.bonuses.iter().any(|b| b.bonus == BonusName::WALLHACK) { return false; }
        let (v1, v2) = checker.edges[i];
        let (fit1, fit2) = (pt_in_poly(pose.vertices[v1], &problem.hole),
                            pt_in_poly(pose.vertices[v2], &problem.hole));
        if fit1 == fit2 { return false; }
        let newhack = if fit1 { v2 } else { v1 };
        match wallhack {
            None => wallhack = Some(newhack),
            Some(a) => {
                if a != newhack { return false; }
            }
        }
    }
    true
}

pub fn globalist_check_edge_lens(problem: &Problem, pose: &Pose, edge_statuses: &[EdgeStatus]) -> bool {
    todo!();
    // let mut eps = 0.;
    // for e in edge_statuses {
    //     eps += f64::abs(e.actual_length as f64 * 4. / e.original_length_x4 as f64 - 1.); 
    // }
    // let edges = problem.figure.edges.len() + pose.vertices.len() - problem.figure.vertices.len();
    // eps * 1e6 <= edges as f64 * problem.epsilon as f64
}

pub fn check_edge_lens(problem: &Problem, pose: &Pose, edge_statuses: &[EdgeStatus]) -> bool {
    if pose.bonuses.iter().any(|b| b.bonus == BonusName::GLOBALIST) {
        return globalist_check_edge_lens(problem, pose, edge_statuses);
    }
    let mut cnt = 0;
    for e in edge_statuses {
        if e.min_length > e.actual_length 
            || e.actual_length > e.max_length { cnt += 1; }
    }
    cnt == 0 || cnt == 1 && pose.bonuses.iter().any(|b| b.bonus == BonusName::SUPERFLEX)
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
        edge_statuses.push(es);
    }
    valid = valid && check_edge_lens(problem, pose, &edge_statuses);
    valid = valid && check_edges_in_hole(problem, pose, &edge_statuses, &checker);

    for i in 0..pose.bonuses.len() {
        for j in 0..pose.bonuses.len() {
            if i == j { continue; }
            if pose.bonuses[i].bonus == pose.bonuses[j].bonus {
                valid = false;
            }
        }
    }

    for _b in &problem.bonuses {
        unlocked.push(false);
    }

    let dislikes = get_dislikes(problem, vertices);

    CheckPoseResponse {
        edges: problem.figure.edges.clone(),  // TODO: break a leg
        edge_statuses,
        dislikes,
        valid,
        unlocked,
        bonus_globalist_sum: None // TODO: globalist
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
