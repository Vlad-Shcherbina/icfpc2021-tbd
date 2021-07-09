#![allow(dead_code)]

use std::path::{Path, PathBuf};
use crate::prelude::{Problem, Pose};

pub fn project_root() -> PathBuf {
    let exe = std::fs::canonicalize(std::env::args().next().unwrap()).unwrap();
    let mut path: &Path = &exe;
    while !(path.file_name().unwrap() == "icfpc2021-tbd" && path.is_dir()) {
        path = path.parent().unwrap();
    }
    path.to_owned()
}

pub fn project_path(rel: impl AsRef<Path>) -> PathBuf {
    // Can't simply return project_root().join(rel)
    // Need to deal with forward and backward slashes on Windows.
    let mut result = project_root();
    for part in rel.as_ref().iter() {
        result.push(part);
    }
    result
}

#[test]
fn project_path_test() {
    assert!(project_path("src/util.rs").exists());
}

pub fn load_problem<S: AsRef<str>>(problem_id: S) -> Problem {
    let path = project_path(format!("data/problems/{}.problem", problem_id.as_ref()));
    let data = std::fs::read(path).unwrap();
    let problem: Problem = serde_json::from_slice(&data).unwrap();
    problem
}

pub fn store_solution<S: AsRef<str>>(problem_id: S, solution: &Pose) {
    let path = format!("outputs/sol_{}.json", problem_id.as_ref());
    let data = serde_json::to_vec(&solution).unwrap();
    std::fs::write(project_path(&path), data).unwrap();
    eprintln!("solution saved to {}", path);
}