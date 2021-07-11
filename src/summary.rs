use std::fmt::Write;
use crate::prelude::*;

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
    writeln!(s, "</tr>").unwrap();

    for problem_id in all_problem_ids() {
        writeln!(s, "<tr>").unwrap();
        writeln!(s, r#"<td class=num>
            <a href="https://poses.live/problems/{}">{}</a>
            <a href="http://127.0.0.1:8000/src/viz/static/viz.html#{}">vis</a>
            </td>"#,
            problem_id, problem_id, problem_id).unwrap();
        writeln!(s, "</tr>").unwrap();
    }
    writeln!(s, "</table>").unwrap();

    let loc = "outputs/summary.html";
    std::fs::write(project_path(loc), s).unwrap();
    println!("see {}", loc);
}