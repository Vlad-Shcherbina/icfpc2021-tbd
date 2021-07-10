use rand::Rng;
use crate::checker::{length_range, check_pose};
use crate::geom::segment_in_poly;
use crate::prelude::*;
use crate::shake::ShakeRequest;

pub fn greedy_shake(r: &ShakeRequest) -> Vec<Pt> {
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

    let expand = |vs: &mut Vec<Pt>| {
        let pr = check_pose(&r.problem, &vs);
        assert!(pr.valid);
        let mut prev_dislikes = pr.dislikes;
        let mut cur_dislikes = pr.dislikes;
        loop {
            //dbg!(cur_dislikes);
            for idx in selected_idxs.iter() {
                //dbg!(idx);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 { continue };
                        let cur = vs[*idx];
                        vs[*idx] = Pt{x: cur.x + dx, y: cur.y + dy};
                        let pr = check_pose(&r.problem, &vs);
                        if pr.valid && pr.dislikes < cur_dislikes {
                            cur_dislikes = pr.dislikes;
                        } else {
                            vs[*idx] = cur;
                        }
                    }
                }
            }
            if cur_dislikes == prev_dislikes {
                break
            } else {
                prev_dislikes = cur_dislikes;
            }
        }
    };

    let shake = |vs: &mut Vec<Pt>, rng:  &mut dyn rand::RngCore| -> i64 {
        let pr = check_pose(&r.problem, &vs);
        assert!(pr.valid);
        let mut cur_dislikes = pr.dislikes;
        for _ in 0..1 {
            for idx in selected_idxs.iter() {
                //dbg!(idx);
                let dx = rng.gen_range(-1..=1);
                let dy = rng.gen_range(-1..=1);
                if dx == 0 && dy == 0 { continue };
                let cur = vs[*idx];
                vs[*idx] = Pt{x: cur.x + dx, y: cur.y + dy};
                let pr = check_pose(&r.problem, &vs);
                // Here we allow perturbations that don't reduce dislikes.
                if pr.valid && pr.dislikes <= cur_dislikes {
                    cur_dislikes = pr.dislikes;
                } else {
                    vs[*idx] = cur;
                }
            }
        }
        return cur_dislikes;
    };

    let mut cur_vs = r.vertices.clone();

    let pr = check_pose(&r.problem, &cur_vs);
    if !pr.valid {
        dbg!("invalid pose passed to greedy shake");
        return cur_vs;
    }
    let mut dislikes = pr.dislikes;
    let convergence_cutoff = r.param*100;
    let mut i = 0;
    loop {
        //dbg!(i);
        expand(&mut cur_vs);
        //dbg!("Shake");
        let cur_dislikes = shake(&mut cur_vs, &mut rng);
        if cur_dislikes < dislikes {
            dislikes = cur_dislikes;
            i = 0;
        } else {
            i += 1;
            if i > convergence_cutoff {
                break;
            }
        }
    }

    cur_vs
}
