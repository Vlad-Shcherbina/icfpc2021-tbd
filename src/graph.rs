

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