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
    pub valid: bool,
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

pub fn check_pose(problem: &Problem, vertices: &[Pt]) -> CheckPoseResponse {
    assert_eq!(problem.figure.vertices.len(), vertices.len());

    let mut edge_statuses = vec![];
    let mut valid = true;
    for &(start, end) in &problem.figure.edges {
        let pt1 = vertices[start];
        let pt2 = vertices[end];

        let fits_in_hole = segment_in_poly((pt1, pt2), &problem.hole);

        let orig_d2 = problem.figure.vertices[start].dist2(problem.figure.vertices[end]);
        let (min_length, max_length) = length_range(orig_d2, problem.epsilon);

        let es = EdgeStatus {
            fits_in_hole,
            actual_length: pt1.dist2(pt2),
            min_length,
            max_length,
        };
        valid = valid && es.is_valid();
        edge_statuses.push(es);
    }

    let mut dislikes = 0;
    for &h in &problem.hole {
        dislikes += vertices.iter().map(|v| v.dist2(h)).min().unwrap();
    }

    CheckPoseResponse {
        edge_statuses,
        dislikes,
        valid,
    }
}

#[test]
fn test_check_pose() {
    let p = crate::util::load_problem("1");
    dbg!(check_pose(&p, &p.figure.vertices));
}
