use std::collections::HashMap;
use std::fmt::Write;
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
    writeln!(s, "<th class=diag><div>hole area</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>unlocks bonuses</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>gets bonuses</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>best solution</div></th>").unwrap();
    writeln!(s, "<th class=diag><div>latest solution</div></th>").unwrap();
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

        let weight = p.figure.vertices.len() * p.figure.edges.len() * p.hole.len();
        let weight = (weight as f64 / 6.0).log2() * 1000.0;
        let weight = weight.round();
        writeln!(s, "<td class=num>{}</td>", weight).unwrap();

        writeln!(s, "<td class=num>{}</td>", p.epsilon).unwrap();

        let bb = bounding_box(&p.hole).unwrap();
        let bb_size = bb.1 - bb.0;
        writeln!(s, "<td class=num>{} x {}</td>", bb_size.x, bb_size.y).unwrap();

        let mut area = 0;
        for x in bb.0.x..=bb.1.x {
            for y in bb.0.y..=bb.1.y {
                if pt_in_poly(Pt::new(x, y), &p.hole) {
                    area += 1;
                }
            }
        }
        writeln!(s, "<td class=num>{}</td>", area).unwrap();

        writeln!(s, "<td class=num>").unwrap();
        for b in p.bonuses {
            writeln!(s, "{:?} for {} <br>", b.bonus, b.problem).unwrap();
        }
        writeln!(s, "</td>").unwrap();

        writeln!(s, "<td class=num>").unwrap();
        for b in bonuslist.get(&problem_id).unwrap_or(&vec![]) {
            writeln!(s, "{}<br>", b).unwrap();
        }
        writeln!(s, "</td>").unwrap();

        let mut scraper = Scraper::new();
        let pi = scraper.problem_info(problem_id);

        let best = match pi.highscore() {
            Some(PoseInfo{id, er}) => match er {
                  EvaluationResult::Invalid => format!(r#"❌, <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}">{}</a>"#, problem_id, id, id),
                  EvaluationResult::Pending => format!(r#"⏳, <a href="http://127.0.0.1:8000/src/viz/static/viz.ht    ml#{}@{}">{}</a>"#, problem_id, id, id),
                  EvaluationResult::Valid{dislikes} => format!(r#"{}, <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}">{}</a>"#, dislikes, problem_id, id, id),
            },
            None => "-".to_string(),
        };

        let latest = match pi.latest() {
            Some(PoseInfo{id, er}) => match er {
                  EvaluationResult::Invalid => format!(r#"❌, <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}">{}</a>"#, problem_id, id, id),
                  EvaluationResult::Pending => format!(r#"⏳, <a href="http://127.0.0.1:8000/src/viz/static/viz.ht    ml#{}@{}">{}</a>"#, problem_id, id, id),
                  EvaluationResult::Valid{dislikes} => format!(r#"{}, <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}@{}">{}</a>"#, dislikes, problem_id, id, id),
            },
            None => "-".to_string(),
        };

        writeln!(s, "<td class=num>{}</td>", best).unwrap();
        writeln!(s, "<td class=num>{}</td>", latest).unwrap();

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
                   .push(format!("{:?} from {}", b.bonus, problem_id));
        }
    }
    bonuses
}
