use std::collections::HashMap;
use std::convert::TryInto;
use crate::domain_model::{BonusName, UnlockedBonus};
use crate::prelude::*;
use crate::geom::{pt_in_poly, segment_in_poly, BBox};
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
    pub bonus_globalist_sum: Option<f64>
}

#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct EdgeStatus {
    pub fits_in_hole: bool,
    pub actual_length: i64,
    pub original_length_x4: i64,
    pub min_length: i64,
    pub max_length: i64,
}

// inclusive
pub fn length_range(d: i64, eps: i64) -> (i64, i64, i64) {
    let min_length = (d * (EPS_BASE - eps) + EPS_BASE - 1) / EPS_BASE;
    let max_length = d * (EPS_BASE + eps) / EPS_BASE;
    (min_length, max_length, d * 4)
}

// Precomputed data to quickly check pose constraints.
pub struct Checker {
    pub problem: Problem,
    pub bbox: BBox,
    pub edge_ranges: Vec<(i64, i64, i64)>, // min, max, orig_x4
    pub edges: Vec<(usize, usize)>,
    pub edge_cache: HashMap<[i16; 4], bool>,
    pub neighbours_cache: HashMap<usize, Vec<usize>>,
    pub bonus: Option<PoseBonus>
}

impl Checker {
    pub fn new(p: &Problem, used_bonuses: &[PoseBonus]) -> Checker {
        let edge_ranges = p.figure.edges.iter()
            .map(|&(start, end)| {
                let d = p.figure.vertices[start].dist2(p.figure.vertices[end]);
                length_range(d, p.epsilon)
            }).collect();
        
        let bonus = if used_bonuses.is_empty() { None } else { Some(used_bonuses[0].clone()) };
        let mut checker = Checker {
            problem: p.clone(),
            edges: p.figure.edges.clone(),
            bbox: BBox::from_pts(&p.hole),
            edge_ranges,
            edge_cache: HashMap::new(),
            neighbours_cache: HashMap::new(),
            bonus: bonus.clone(),
        };

        if used(&bonus, &BonusName::BREAK_A_LEG) && check_valid_break_a_leg(&bonus.as_ref().unwrap(), p) {
            let (v1, v2) = bonus.unwrap().edge.unwrap();
            let idx = checker.edges.iter().position(|&(p1, p2)| 
                v1 == p1 && v2 == p2 || v1 == p2 && v2 == p1
            ).unwrap();
            checker.edges.remove(idx);
            checker.edge_ranges.remove(idx);
            checker.edges.push((v1, p.figure.vertices.len()));
            checker.edges.push((v2, p.figure.vertices.len()));
            let p1x = p.figure.vertices[v1].x * 4;
            let p1y = p.figure.vertices[v1].y * 4;
            let p2x = p.figure.vertices[v2].x * 4;
            let p2y = p.figure.vertices[v2].y * 4;
            
        }

        checker
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

pub fn used(pb: &Option<PoseBonus>, bn: &BonusName) -> bool {
    if let Some(PoseBonus { bonus: b, problem: _, edge: _ }) = pb { *b == *bn } else { false }
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

pub fn list_unlocked_bonuses(problem: &Problem, vertices: &[Pt]) -> Vec<UnlockedBonus> {
    let unlocked = check_unlocked(problem, vertices);
    unlocked.iter().zip(&problem.bonuses)
    .filter_map(|(u, b)| if *u {
        Some( UnlockedBonus {
            problem_id: b.problem,
            name: b.bonus,
        })
    } else {
        None
    })
    .collect()
}


#[allow(clippy::needless_range_loop)]
pub fn check_edges_in_hole(problem: &Problem, pose: &Pose, 
        edge_statuses: &[EdgeStatus], checker: &Checker) -> bool {
    let mut wallhack: Option<usize> = None;
    for i in 0..edge_statuses.len() {
        if edge_statuses[i].fits_in_hole { continue; }
        if !used(&checker.bonus, &BonusName::WALLHACK) { return false; }
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

pub fn check_valid_break_a_leg(bonus: &PoseBonus, problem: &Problem) -> bool {
    match bonus.edge {
        None => false,
        Some((v1, v2)) => {
            problem.figure.edges.iter().any(|&(p1, p2)| 
                v1 == p1 && v2 == p2 || v1 == p2 && v2 == p1
            )
        }
    }
}

pub fn globalist_sum_len(edge_statuses: &[EdgeStatus]) -> f64 {
    let mut eps = 0.;
    for e in edge_statuses {
        eps += f64::abs(e.actual_length as f64 * 4. / e.original_length_x4 as f64 - 1.); 
    }
    eps * 1e6
}

pub fn globalist_check_edge_lens(problem: &Problem, sum_eps: f64) -> bool {
    sum_eps <= problem.figure.edges.len() as f64 * problem.epsilon as f64
}

pub fn no_glob_check_edge_lens(pose: &Pose, edge_statuses: &[EdgeStatus]) -> bool {
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

        let (min_length, max_length, original_length_x4) = checker.edge_ranges[i];

        let es = EdgeStatus {
            fits_in_hole,
            actual_length: pt1.dist2(pt2),
            original_length_x4,
            min_length,
            max_length,
        };
        edge_statuses.push(es);
    }

    let bonus_globalist_sum = if used(&checker.bonus, &BonusName::GLOBALIST) {
        Some(globalist_sum_len(&edge_statuses))
    }
    else { None };
    valid = valid && if used(&checker.bonus, &BonusName::GLOBALIST) {
        globalist_check_edge_lens(problem, bonus_globalist_sum.unwrap())
    }
    else {
        no_glob_check_edge_lens(pose, &edge_statuses)
    };

    valid = valid && check_edges_in_hole(problem, pose, &edge_statuses, &checker);
    valid = valid && pose.bonuses.len() <= 1;

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
        bonus_globalist_sum
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
