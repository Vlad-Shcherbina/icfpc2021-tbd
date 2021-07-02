#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub fn project_root() -> PathBuf {
    let exe = std::fs::canonicalize(std::env::args().next().unwrap()).unwrap();
    let mut path: &Path = &exe;
    while path.file_name().unwrap() != "icfpc2021-tbd" {
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
