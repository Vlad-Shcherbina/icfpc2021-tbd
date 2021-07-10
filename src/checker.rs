use crate::prelude::*;
use crate::geom::segment_in_poly;

#[derive(serde::Deserialize)]
pub struct CheckPoseRequest {
    pub problem: Problem,
    pub vertices: Vec<Pt>,
}

#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct CheckPoseResponse {
    pub edge_statuses: Vec<EdgeStatus>,
    pub dislikes: i64,
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

pub fn check_pose(problem: &Problem, vertices: &[Pt]) -> CheckPoseResponse {
    assert_eq!(problem.figure.vertices.len(), vertices.len());

    let mut edge_statuses = vec![];
    for &(start, end) in &problem.figure.edges {
        let pt1 = vertices[start];
        let pt2 = vertices[end];

        let fits_in_hole = segment_in_poly((pt1, pt2), &problem.hole);

        let orig_d2 = problem.figure.vertices[start].dist2(problem.figure.vertices[end]);
        let (min_length, max_length) = length_range(orig_d2, problem.epsilon);

        edge_statuses.push(EdgeStatus {
            fits_in_hole,
            actual_length: pt1.dist2(pt2),
            min_length,
            max_length,
        });
    }

    CheckPoseResponse {
        edge_statuses,
        dislikes: 42,
    }
}

#[test]
fn test_check_pose() {
    let p = crate::util::load_problem("1");
    dbg!(check_pose(&p, &p.figure.vertices));
}
