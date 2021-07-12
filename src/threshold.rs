#![allow(unused_imports)]

use rand::Rng;
use crate::checker::{length_range, check_pose, get_dislikes};
use crate::geom::*;
use crate::prelude::*;
use crate::graph::*;
use crate::shake::ShakeRequest;
use rand::prelude::SliceRandom;
use ndarray::Array2;

pub struct HoleChecker {
    x_min: i64,
    x_max: i64,
    y_min: i64,
    y_max: i64,
    mask: Array2<i16>,
    poly: Vec<Pt>
}

fn get_hole_mask(problem: &Problem) -> Array2<i16> {
    let (_, pt_max) = bounding_box(&problem.hole).unwrap();
    let xdim = pt_max.x + 1;
    let ydim = pt_max.y + 1;
    let mut result = Array2::ones((xdim as usize, ydim as usize));
    for x in 0..xdim {
        for y in 0..ydim {
            if pt_in_poly(Pt::new(x, y), &problem.hole) {
                result[[x as usize, y as usize]] = 0;
            }
        }
    }
    result
}

impl HoleChecker {
    pub fn new(problem: &Problem) -> HoleChecker {
        let (pt_min, pt_max) = bounding_box(&problem.hole).unwrap();
        HoleChecker {
            x_min: pt_min.x,
            x_max: pt_max.x,
            y_min: pt_min.y,
            y_max: pt_max.y,
            mask: get_hole_mask(problem),
            poly: problem.hole.clone()
        }
    }
    pub fn coord_in_hole(&self, x: i64, y: i64) -> bool {
        if (self.x_min <= x) && (x <= self.x_max) && (self.y_min <= y) && (y <= self.y_max) {
            self.mask[[x as usize, y as usize]] == 0
        } else {
            false
        }
    }
    //fn pt_in_hole(&self, pt: Pt) -> bool {
    //    self.coord_in_hole(pt.x, pt.y)
    //}
    pub fn segment_in_hole(&self, pt1: Pt, pt2: Pt) -> bool {
        segment_in_poly((pt1, pt2), &self.poly)
    }
}

pub fn orig_distance(problem: &Problem, v1_id: usize, v2_id: usize) -> i64 {
    problem.figure.vertices[v1_id].dist2(problem.figure.vertices[v2_id])
}

pub fn deformation_limits(problem: &Problem, v1_id: usize, v2_id: usize) -> (i64, i64) {
    let orig_d2 = orig_distance(problem, v1_id, v2_id);
    let (min_d, max_d, _) = crate::checker::length_range(orig_d2, problem.epsilon);
    (min_d, max_d)
}

struct BBox {
    x_min: i64,
    x_max: i64,
    y_min: i64,
    y_max: i64
}

impl BBox {
    fn intersect(&self, other: &BBox) -> BBox {
        BBox {
            x_min: self.x_min.max(other.x_min),
            x_max: self.x_max.min(other.x_max),
            y_min: self.y_min.max(other.y_min),
            y_max: self.y_max.min(other.y_max)
        }
    }
    fn is_empty(&self) -> bool {
        (self.x_max < self.x_min) || (self.y_max < self.y_min)
    }
}

// A bbox constraining valid positions of vertice idx based on position of neighbour.
fn neighbour_valid_bbox(problem: &Problem, vs: &mut Vec<Pt>, idx: usize, neighbour: usize) -> BBox {
    let (_, max_dist2) = deformation_limits(problem, idx, neighbour);
    let max_len = ((max_dist2 + 1) as f64).sqrt() as i64;
    //dbg!(idx, neighbour, max_len);
    //dbg!(vs[neighbour]);
    let (x, y) = (vs[neighbour].x, vs[neighbour].y);
    BBox {
        x_min: x - max_len,
        x_max: x + max_len,
        y_min: y - max_len,
        y_max: y + max_len
    }
}


// A bbox constraining valid positions of verice idx based on position of all neighbours.
fn valid_positions_bbox(problem: &Problem, vs: &mut Vec<Pt>, idx: usize) -> BBox {
    let neighbours: Vec<_> = neighbours(&problem.figure.edges, idx).collect();
    let mut bbox = neighbour_valid_bbox(problem, vs, idx, neighbours[0]);

    for neighbour in neighbours {
        bbox = bbox.intersect(&neighbour_valid_bbox(problem, vs, idx, neighbour));
        //dbg!(bbox.x_min, bbox.x_max, bbox.y_min, bbox.y_max);
        if bbox.is_empty() {
            break;
        }
    }
    bbox
}

pub fn valid_positions(problem: &Problem, vs: &mut Vec<Pt>, idx: usize, hole_checker: &HoleChecker) -> Vec<Pt> {
    let neighbours: Vec<_> = neighbours(&problem.figure.edges, idx).collect();
    let (pt_min, pt_max) = bounding_box(&problem.hole).unwrap();
    let mut result = vec![];

    let hole_bbox = BBox {
        x_min: pt_min.x,
        x_max: pt_max.x,
        y_min: pt_min.y,
        y_max: pt_max.y
    };

    let bbox = hole_bbox.intersect(&valid_positions_bbox(problem, vs, idx));

    //dbg!(hole_bbox.x_min, hole_bbox.x_max, hole_bbox.y_min, hole_bbox.y_max);
    //dbg!(idx, bbox.x_min, bbox.x_max, bbox.y_min, bbox.y_max);
    for x in bbox.x_min..=bbox.x_max {
        'l: for y in bbox.y_min..=bbox.y_max {
            if !hole_checker.coord_in_hole(x, y) {
                continue;
            }
            let assumed_pos = Pt { x, y };
            for neighbour_idx in &neighbours {
                let neighbour = vs[*neighbour_idx];
                let (min_dist, max_dist) = deformation_limits(problem, idx, *neighbour_idx);
                let new_dist = assumed_pos.dist2(neighbour);
                if !(min_dist <= new_dist && new_dist <= max_dist) {
                    // eprintln!("Drop {:?} by distance", assumed_pos);
                    continue 'l;
                }

                //if !((x >= x_min) && (x <= x_max) && (y >= y_min) && (y <= y_max)) {
                //    dbg!(x_min, x, x_max);
                //    dbg!(y_min, y, y_max);
                //}
                //assert!((x >= x_min) && (x <= x_max) && (y >= y_min) && (y <= y_max));
                if !hole_checker.segment_in_hole(neighbour, assumed_pos) {
                    // eprintln!("Drop {:?} by intersection", assumed_pos);
                    continue 'l;
                }
            }
            result.push(assumed_pos);
        }
    }
    result
}

fn step(problem: &Problem, vs: &mut Vec<Pt>, selected_idxs: &[usize], rng:  &mut dyn rand::RngCore, hole_checker: &HoleChecker, threshold: i64) {
    let cur_dislikes = get_dislikes(problem, vs);
    let mut selected_idxs_shuffled = selected_idxs.to_vec();
    selected_idxs_shuffled.shuffle(rng);

    for idx in selected_idxs_shuffled.iter() {
        //dbg!(idx);
        let mut acceptable_perturbations = vec![];
        let cur = vs[*idx];
        for pt in valid_positions(problem, vs, *idx, hole_checker) {
            if pt == vs[*idx] {
                continue;
            }
            vs[*idx] = pt;
            let dislikes = get_dislikes(problem, vs);
            if dislikes - cur_dislikes <= threshold {
                acceptable_perturbations.push(pt);
            }
        }
        if acceptable_perturbations.is_empty() {
            //dbg!("empty");
            vs[*idx] = cur;
            // continue;
        } else {
            //dbg!("non-empty");
            vs[*idx] = *acceptable_perturbations.choose(rng).unwrap();
            return;
        }

    }
}

fn threshold(i: i64, param: i64) -> i64 {
    //dbg!(i, param);
    let nz = 10000;
    if i < nz {
        (0.05 * param as f64 * (1.0 - (i as f64 / nz as f64))) as i64
    } else {
        0
    }
}

pub fn threshold_shake(r: &ShakeRequest) -> Vec<Pt> {
    //assert!(r.problem.bonuses.is_empty());
    //dbg!(r.problem.figure.vertices.len(), r.problem.hole.len());
    let mut selected = r.selected.clone();
    if selected.iter().all(|&s| !s) {
        selected = vec![true; selected.len()];
    }
    let mut selected_idxs = vec![];
    for (i, &sel) in selected.iter().enumerate() {
        if sel {
            selected_idxs.push(i);
        }
    }
    let mut rng = rand::thread_rng();
    
    let hole_checker = HoleChecker::new(&r.problem);

    let mut cur_vs = r.vertices.clone();

    let pr = check_pose(&r.problem, &Pose { vertices: cur_vs.clone(), bonuses: vec![] });
    if !pr.valid {
        dbg!("invalid pose passed to threshold shake");
        return cur_vs;
    }
    let mut dislikes = pr.dislikes;
    let convergence_cutoff = r.param*50;
    let mut j = 0;
    for i in 0.. {
        // dbg!(i);
        let threshold = threshold(i, pr.dislikes);
        step(&r.problem, &mut cur_vs, &selected_idxs, &mut rng, &hole_checker, threshold);

        let cur_dislikes = get_dislikes(&r.problem, &cur_vs);
        //if threshold > 0 { dbg!(threshold); }
        // dbg!(cur_dislikes);
        if cur_dislikes != dislikes {
            dislikes = cur_dislikes;
            j = 0;
        } else {
            j += 1;
            if j > convergence_cutoff {
                break;
            }
        }
    }

    cur_vs
}
