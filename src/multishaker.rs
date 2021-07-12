use rand::Rng;
//use rand::seq::SliceRandom;
use crate::prelude::*;
use crate::checker::{Checker, get_dislikes, check_pose};
use crate::submitter::Submitter;
use crate::daiquiri;
use crate::shake::ShakeRequest;
use crate::geom::pt_in_poly;
use crate::greedy;
use crate::threshold;

fn tuck(p: &Problem, pts: &mut [Pt], checker: &mut Checker, rng: &mut rand::RngCore) -> bool {
    loop {
        eprintln!("Tucking vertices...");
        let mut vertices_inside = vec![];
        let mut vertices_outside = vec![];
        for (idx, pt) in pts.iter().enumerate() {
            if pt_in_poly(*pt, &p.hole) {
                vertices_inside.push(idx);
            } else {
                vertices_outside.push(idx);
            }
        }
        assert!(!vertices_inside.is_empty());
        let pt_inside = pts[vertices_inside[rng.gen_range(0..vertices_inside.len())]];
        if (!vertices_outside.is_empty()) {
            eprintln!("Vertices found.");
            for idx_outside in vertices_outside {
                pts[idx_outside] = pt_inside;
                eprintln!("moving {} to {}", idx_outside, idx_outside);
            }
        }

        // No new points will appear on the ouside now. But new offending edges might appear.
        loop {
            let pose = Pose{vertices: pts.to_vec(), bonuses: vec![]};
            let response = check_pose(p, &pose);
            if response.valid {
                eprintln!("valid");
                return true;
            }
            let bad_edges: Vec<usize> = (0..response.edges.len()).filter(|e| !response.edge_statuses[*e].fits_in_hole).collect();
            if bad_edges.is_empty() {
                eprintln!("no bad edges");                
                return false;
            }
            eprintln!("Tucking edges...");
            for edge in bad_edges {
                eprintln!("moving {}", edge);
                let (idx1, idx2) = response.edges[edge];
                pts[idx1] = pt_inside;
                pts[idx2] = pt_inside;
            }
        }
    }
}

crate::entry_point!("multishaker", multishaker);
fn multishaker() {
    let problem_id: i32 = std::env::args().nth(2).unwrap().parse().unwrap();

    let mut submitter = Submitter::new(problem_id);

    
    let p = load_problem(problem_id);
    let bonuses = vec![];

    let mut checker = Checker::new(&p, &bonuses);
    let mut rng = rand::thread_rng();

    let mut pts = p.figure.vertices.clone();

    // Make valid pose with daquiri.
    loop {
        eprintln!("Tuck/mojito...");
        let valid = tuck(&p, &mut pts, &mut checker, &mut rng);
        if valid {
            break;
        }
        
        let mut stabilized = false;
        while !stabilized {
            eprintln!("Mojito loop...");
            let request = ShakeRequest {
                problem: p.clone(),
                vertices: pts.clone(),
                selected: vec![true; pts.len()],
                method: "mojito".to_string(),
                param: 5,
            };
            let new_pts = daiquiri::daikuiri_shake(&request, true);
            stabilized = (new_pts == pts);
            pts = new_pts;
        }
    }

    'outer: loop {
        if submitter.try_submit() {
            // Submitted best possible solution. Nothing to do.
            return;
        }


        // Use greedy shaker.
        let request = ShakeRequest {
            problem: p.clone(),
            vertices: pts.clone(),
            selected: vec![true; pts.len()],
            method: "greedy".to_string(),
            param: 2,
        };
        pts = greedy::greedy_shake(&request);
        let pose = Pose{vertices: pts.clone(), bonuses: vec![]};
        submitter.update(get_dislikes(&p, &pts), &pose);

        let request = ShakeRequest {
            problem: p.clone(),
            vertices: pts.clone(),
            selected: vec![true; pts.len()],
            method: "threshold".to_string(),
            param: 2,
        };
        pts = threshold::threshold_shake(&request);
        let pose = Pose{vertices: pts.clone(), bonuses: vec![]};
        submitter.update(get_dislikes(&p, &pts), &pose);

        // Use threshold shaker.
    }
}
