use crate::prelude::*;
use crate::geom::rotate_point;

#[derive(serde::Deserialize)]
pub struct RotateRequest {
    pub problem: Problem,
    pub vertices: Vec<Pt>,
    pub selected: Vec<bool>,
    pub pivot: Option<Pt>,
    pub angle: i16,
}

pub fn rotate(r: &RotateRequest) -> Vec<Pt> {
    match r.pivot {
        None => center_of_mass_rotation(r),
        Some(pivot) => pivot_rotation(r, pivot),
    }
}

pub fn center_of_mass_rotation(r: &RotateRequest) -> Vec<Pt> {
    let mut vs = r.vertices.clone();
    assert_eq!(vs.len(), r.selected.len());

    let sum_x : i64 = vs.iter().map(|pt| pt.x).sum();
    let sum_y : i64 = vs.iter().map(|pt| pt.y).sum();
    let com_x = sum_x as f64 / vs.len() as f64;
    let com_y = sum_y as f64 / vs.len() as f64;

    let com_pivot = Pt::new(com_x.round() as i64, com_y.round() as i64);

    for (v, &sel) in vs.iter_mut().zip(r.selected.iter()) {
        if sel {
            *v = rotate_point(*v, com_pivot, r.angle)
        }
    }
    vs

}

pub fn pivot_rotation(r: &RotateRequest, pivot: Pt) -> Vec<Pt> {
    let mut vs = r.vertices.clone();
    assert_eq!(vs.len(), r.selected.len());

    for (v, &sel) in vs.iter_mut().zip(r.selected.iter()) {
        if sel {
            *v = rotate_point(*v, pivot, r.angle)
        }
    }
    vs


}
