use rand::Rng;
use crate::prelude::*;

#[derive(serde::Deserialize)]
pub struct ShakeRequest {
    pub problem: Problem,
    pub vertices: Vec<Pt>,
    pub selected: Vec<bool>,
    pub method: String,
    pub param: i64,
}

pub fn shake(r: &ShakeRequest) -> Vec<Pt> {
    match r.method.as_str() {
        "random" => random_shake(r),
        "banana" => crate::banana::banana_shake(r),
        "ice" => todo!(),
        s => panic!("{:?}", s),
    }
}

fn random_shake(r: &ShakeRequest) -> Vec<Pt> {
    let mut rng = rand::thread_rng();

    let mut vs = r.vertices.clone();
    assert_eq!(vs.len(), r.selected.len());
    for (v, &sel) in vs.iter_mut().zip(r.selected.iter()) {
        if sel {
            v.x += rng.gen_range(-r.param..=r.param);
            v.y += rng.gen_range(-r.param..=r.param);
        }
    }
    vs
}
