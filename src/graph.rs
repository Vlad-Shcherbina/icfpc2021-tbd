use std::collections::{HashSet, VecDeque};

pub fn neighbours(edges: &[(usize, usize)], v_id: usize) -> impl Iterator<Item=usize> + '_ {
    edges.iter().filter_map(move |(from, to)| {
        if *from == v_id {
            Some(*to)
        } else if *to == v_id {
            Some(*from)
        } else {
            None
        }
    })
}

pub fn bfs(edges: &[(usize, usize)], v_id: usize) -> Vec<usize> {
    let mut queue = VecDeque::<usize>::new();
    let mut visited: HashSet<usize> = Default::default();
    visited.insert(v_id);
    queue.push_back(v_id);
    let mut result = vec![];
    while !queue.is_empty() {
        let current_vid = queue.pop_front().unwrap();
        result.push(current_vid);
        let ns: Vec<_> = neighbours(edges, current_vid).collect();
        for n_id in ns {
            if !visited.contains(&n_id) {
                queue.push_back(n_id);
                visited.insert(n_id);
            }
        }
    }
    result
}