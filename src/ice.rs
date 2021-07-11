#![allow(unused_imports)]

use rand::Rng;
use crate::checker::Checker;
use crate::geom::segment_in_poly;
use crate::prelude::*;
use crate::shake::ShakeRequest;

pub fn ice_shake(r: &ShakeRequest) -> Vec<Pt> {
    let mut pose = Pose {
        bonuses: vec![],
        vertices: r.vertices.clone(),
    };

    let mut checker = Checker::new(&r.problem, &pose.bonuses);

    let mut selected = r.selected.clone();
    if selected.iter().all(|&s| !s) {
        selected = vec![true; selected.len()];
    }

    let selected_idx: Vec<usize> = selected.iter().enumerate()
        .filter_map(|(i, &sel)| if sel { Some(i) } else { None })
        .collect();

    let edges = checker.edges.clone();
    let mut inci: Vec<Vec<usize>> = vec![vec![]; edges.len()];
    for (i, &(start, end)) in edges.iter().enumerate() {
        inci[start].push(i);
        inci[end].push(i);
    }

    let mut edge_infos: Vec<EdgeInfo> = edges.iter().enumerate()
        .map(|(i, &(start, end))| {
            EdgeInfo::new(&mut checker, i, pose.vertices[start], pose.vertices[end])
        }).collect();
    let mut e_total = EdgeInfo::zero();
    for &ei in &edge_infos {
        e_total = e_total + ei;
    }
    dbg!(e_total);

    let mut score = e_total.hole_penalty as f64 + e_total.length_penalty;

    let mut best_pose = pose.clone();
    let mut best_score = score;
    let mut best_edge_infos = edge_infos.clone();
    let mut best_e_total = e_total;

    let mut rng = rand::thread_rng();
    let num_steps = 500_000;
    for step in 0..num_steps {
        let threshold = (num_steps - step) as f64 / num_steps as f64;
        if step % (num_steps / 10 + 1) == 0 {
            eprintln!("{} {} {}", step, threshold, score);
        }

        if rng.gen_range(0..num_steps/10) == 0 {
            eprintln!("reset @ {}", step);
            score = best_score;
            pose = best_pose.clone();
            edge_infos = best_edge_infos.clone();
            e_total = best_e_total;
        }

        assert!(e_total.length_penalty >= 0.0, "{:?}", e_total);

        let v_idx = selected_idx[rng.gen_range(0..selected_idx.len())];
        let old_pt = pose.vertices[v_idx];
        /*let new_pt = Pt {
            x: old_pt.x + rng.gen_range(-1..=1),
            y: old_pt.y + rng.gen_range(-1..=1),
        };*/
        let new_pt = checker.inside[rng.gen_range(0..checker.inside.len())];
        pose.vertices[v_idx] = new_pt;

        let old_e_total = e_total;

        let mut old_infos = vec![];
        for &e_idx in &inci[v_idx] {
            let old_info = edge_infos[e_idx];
            old_infos.push(old_info);
            e_total = e_total - old_info;

            let (start, end) = edges[e_idx];
            let new_info = EdgeInfo::new(&mut checker, e_idx,
                pose.vertices[start],
                pose.vertices[end]);
            e_total = e_total + new_info;
            edge_infos[e_idx] = new_info;
        }

        let new_score = e_total.hole_penalty as f64 + e_total.length_penalty;

        if new_score < score + threshold {
            pose.vertices[v_idx] = old_pt;
            score = new_score;
            if score < best_score {
                best_score = score;
                best_pose = pose.clone();
                best_edge_infos = edge_infos.clone();
                best_e_total = e_total;
                eprintln!("{}, {} {:?}", step, best_score, e_total);
            }
        } else {
            e_total = old_e_total;
            for (&e_idx, &old_info) in inci[v_idx].iter().zip(old_infos.iter()) {
                edge_infos[e_idx] = old_info;
            }
        }
    }
    best_pose.vertices
}

#[derive(Debug, Clone, Copy)]
struct EdgeInfo {
    hole_penalty: i32,
    length_penalty: f64,
}

impl std::ops::Add for EdgeInfo {
    type Output = EdgeInfo;

    fn add(self, rhs: Self) -> Self::Output {
        EdgeInfo {
            hole_penalty: self.hole_penalty + rhs.hole_penalty,
            length_penalty: self.length_penalty + rhs.length_penalty,
        }
    }
}

impl std::ops::Sub for EdgeInfo {
    type Output = EdgeInfo;

    fn sub(self, rhs: Self) -> Self::Output {
        EdgeInfo {
            hole_penalty: self.hole_penalty - rhs.hole_penalty,
            length_penalty: self.length_penalty - rhs.length_penalty,
        }
    }
}

impl EdgeInfo {
    fn zero() -> EdgeInfo {
        EdgeInfo {
            hole_penalty: 0,
            length_penalty: 0.0,
        }
    }
    fn new(checker: &mut Checker, edge_idx: usize, pt1: Pt, pt2: Pt) -> Self {
        let (min_d, max_d) = checker.edge_ranges[edge_idx];
        let d = pt1.dist2(pt2);
        let length_penalty = if d < min_d {
            (min_d - d) as f64 / max_d as f64
        } else if d > max_d {
            (d - max_d) as f64 / max_d as f64
        } else {
            0.0
        };
        assert!(length_penalty >= 0.0);
        EdgeInfo {
            hole_penalty: if checker.edge_in_hole(pt1, pt2) { 0 } else { 1 },
            length_penalty,
        }
    }
}

crate::entry_point!("ice_demo", ice_demo);
fn ice_demo() {
    let p = load_problem(std::env::args().nth(2).unwrap().parse().unwrap());

    let r = ShakeRequest {
        vertices: p.figure.vertices.clone(),
        selected: vec![true; p.figure.vertices.len()],
        method: "ice".to_owned(),
        param: 42,
        problem: p,
    };
    let start = std::time::Instant::now();
    ice_shake(&r);
    eprintln!("it took {} s", start.elapsed().as_secs_f64());
}
