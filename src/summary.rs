use std::collections::HashMap;
use std::fmt::Write;
use EvaluationResult::Valid;

use crate::checker::check_pose;
use crate::prelude::*;
use crate::geom::*;
use crate::poses_live::*;

crate::entry_point!("summary", summary);
fn summary() {
    let mut s = String::new();
    writeln!(s, "{}", r#"
    <style>
    table {
        margin-top: 50px;
    }
    th.diag > div {
        transform: translate(5px, 0px) rotate(-30deg);
        width: 25px;
        white-space: nowrap;
    }
    td {
        border: solid 1px #ccc;
    }
    td.num {
        text-align: right;
    }
    </style>
    "#).unwrap();

    writeln!(s, "<table>").unwrap();
    writeln!(s, "<tr>").unwrap();
    writeln!(s, "<th class=diag><div>problem ID</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>vertices</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>edges</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>hole</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>scoring weight</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>epsilon</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>bounding box</div></th>").unwrap();
    // writeln!(s, "<th class=diag><div>hole area</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>unlocks bonuses</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>gets bonuses</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>best solution</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>latest solution</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>best normalized</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>latest normalized</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>last uses bonuses</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>last unlocks bonuses</div></th>").unwrap();
    writeln!(s, "</tr>").unwrap();

    let bonuslist = get_bonus_list();
    for problem_id in all_problem_ids() {
        dbg!(problem_id);
        let p = load_problem(problem_id);

        writeln!(s, "<tr>").unwrap();
        writeln!(s, r#"<td class=num>
            <a href="https://poses.live/problems/{}">{}</a>
            <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}">vis</a>
            </td>"#,
            problem_id, problem_id, problem_id).unwrap();


        writeln!(s, "<td class=num>{}</td>", p.figure.vertices.len()).unwrap();
        writeln!(s, "<td class=num>{}</td>", p.figure.edges.len()).unwrap();
        writeln!(s, "<td class=num>{}</td>", p.hole.len()).unwrap();

        writeln!(s, "<td class=num>{}</td>", p.weight().round()).unwrap();

        writeln!(s, "<td class=num>{}</td>", p.epsilon).unwrap();

        let bb = bounding_box(&p.hole).unwrap();
        let bb_size = bb.1 - bb.0;
        writeln!(s, "<td class=num>{} x {}</td>", bb_size.x, bb_size.y).unwrap();

        // let mut area = 0;
        // for x in bb.0.x..=bb.1.x {
        //     for y in bb.0.y..=bb.1.y {
        //         if pt_in_poly(Pt::new(x, y), &p.hole) {
        //             area += 1;
        //         }
        //     }
        // }
        // writeln!(s, "<td class=num>{}</td>", area).unwrap();

        writeln!(s, "<td class=num>").unwrap();
        for b in &p.bonuses {
            writeln!(s, "{}=>{}, ", b.bonus.short_name(), b.problem).unwrap();
        }
        writeln!(s, "</td>").unwrap();

        writeln!(s, "<td class=num>").unwrap();
        for b in bonuslist.get(&problem_id).unwrap_or(&vec![]) {
            writeln!(s, "{}, ", b).unwrap();
        }
        writeln!(s, "</td>").unwrap();

        // let mut scraper = Scraper::new();
        // let pi = scraper.problem_info(problem_id);
        let data = read_cache();
        let pi = data.problems.get(&problem_id).unwrap();

        let best = match pi.highscore() {
            Some(PoseInfo{id, er}) =>
                format!(r#"{}, <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}">vis</a>"#,
                    er, problem_id, id),
            None => "-".to_string(),
        };

        let mut latest = String::new();
        let mut used_bonuses = String::new();
        let mut unlocked_bonuses = String::new();

        match pi.latest() {
            None => {
                latest = "-".to_string();
                used_bonuses = "-".to_string();
                unlocked_bonuses = "-".to_string();
            }
            Some(PoseInfo{id, er}) => {
                latest = format!(r#"{}, <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}">vis</a>"#,
                er, problem_id, id);
                match er {
                    Valid { dislikes: _ } => {
                        let pose = data.poses.get(id).unwrap();
                        for b in &pose.bonuses { used_bonuses += &format!("{}=>{}, ", 
                                                b.problem,
                                                b.bonus.short_name()); }
                        let c = check_pose(&p, &pose);
                        for i in 0..c.unlocked.len() {
                            if !c.unlocked[i] { continue; }
                            unlocked_bonuses += &format!("{}=>{}, ", 
                                                p.bonuses[i].bonus.short_name(),
                                                p.bonuses[i].problem);
                        }
                    }
                    _ => {
                        used_bonuses = "-".to_string();
                        unlocked_bonuses = "-".to_string();
                    }
                }           
            }
        }

        writeln!(s, "<td class=num>{}</td>", best).unwrap();
        writeln!(s, "<td class=num>{}</td>", latest).unwrap();

        writeln!(s, "<td class=num>{}</td>", best).unwrap();
        writeln!(s, "<td class=num>{}</td>", latest).unwrap();

        writeln!(s, "<td class=num>{}</td>", used_bonuses).unwrap();
        writeln!(s, "<td class=num>{}</td>", unlocked_bonuses).unwrap();
    }
    writeln!(s, "</table>").unwrap();

    let loc = "outputs/summary.html";
    std::fs::write(project_path(loc), s).unwrap();
    println!("see {}", loc);
}

fn get_bonus_list() -> HashMap<i32, Vec<String>> {
    let mut bonuses: HashMap<i32, Vec<String>> = HashMap::new();
    for problem_id in all_problem_ids() {
        let p = load_problem(problem_id);
        for b in p.bonuses {
            bonuses.entry(b.problem)
                   .or_default()
                   .push(format!("{}=>{}", problem_id, b.bonus.short_name()));
        }
    }
    bonuses
}
