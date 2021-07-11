#![allow(unused_imports)]

use std::io::Write;
use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::util::*;

crate::entry_point!("bonus_graph", bonus_graph);
fn bonus_graph() {
    let mut next: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut fout = std::fs::File::create(project_path("outputs/bonus_graph.dot")).unwrap();
    writeln!(fout, "digraph {{").unwrap();
    writeln!(fout, "  legend_x -> legend_y [label=BREAK_A_LEG,color=blue];").unwrap();
    writeln!(fout, "  legend_y -> legend_z [label=GLOBALIST,color=green];").unwrap();
    writeln!(fout, "  legend_z -> legend_t [label=WALLHACK,color=red];").unwrap();

    for problem_no in 1..=106 {
        let p = load_problem(problem_no.to_string());
        print!("{} -> ", problem_no);

        let e = next.entry(problem_no).or_default();

        for b in &p.bonuses {
            print!("{:?} {}  ", b.bonus, b.problem);
            e.push(b.problem);
            let color = match b.bonus {
                crate::domain_model::BonusName::GLOBALIST => "green",
                crate::domain_model::BonusName::BREAK_A_LEG => "blue",
                crate::domain_model::BonusName::WALLHACK => "red",
            };
            writeln!(fout, "  {} -> {} [color={}];", problem_no, b.problem, color).unwrap();
        }
        println!();
    }

    writeln!(fout, "}}").unwrap();

    std::process::Command::new("neato")
        .arg("-Tpng")
        .arg("-o")
        .arg("outputs/bonus_graph.png")
        .arg("outputs/bonus_graph.dot")
        .spawn()
        .unwrap();

    println!("see outputs/bonus_graph.png");

    // println!("now run the following command:");
    // println!("dot -Tpng -o outputs/bonus_graph.png <outputs/bonus_graph.dot");

    /*let mut cycles: Vec<Vec<u32>> = vec![];
    let mut visited: HashSet<u32> = HashSet::new();
    loop {
        let mut i = match next.keys().find(|p| !visited.contains(p)) {
            Some(i) => *i,
            None => break,
        };
        let mut cycle = vec![];
        let start = i;
        loop {
            visited.insert(i);
            cycle.push(i);
            let nn = &next[&i];
            assert_eq!(nn.len(), 1);
            i = nn[0];
            if i == start {
                break;
            }
        }
        eprintln!("{:?}", cycle);
    }*/
    // let mut i = 1;
    // let mut cnt = 0;
    // loop {
    //     cnt += 1;
    //     dbg!(i);
    //     i = next[&i][0];
    //     if i == 1 {
    //         break;
    //     }
    // }
    // dbg!(cnt);
}
