import { Pt, Pair, Figure, Edge, Problem, World, Frame, Universe, Change, Invalid } from "./types.js"

function drag(cs: Change[], u: Universe): Universe {
    const problem = u.problem;
    const world = u.world;
    var world1 = clone_world(world);
    for (var csi in cs) {
        world1.figure.vertices[cs[csi].id] = [...cs[csi].destination];
    }
    return validate_world(world1, u);
}

function clone_world(w: World): World {
    return {
        hole: clone_array_of_pts(w.hole),
        figure: {
            edges: clone_array_of_pts(w.figure.edges),
            vertices: clone_array_of_pts(w.figure.vertices),
        },
        epsilon: w.epsilon,
    };
}

function clone_problem(p: Problem): Problem {
    return {
        hole: clone_array_of_pts(p.hole),
        figure: {
            edges: clone_array_of_pts(p.figure.edges),
            vertices: clone_array_of_pts(p.figure.vertices),
        },
        epsilon: p.epsilon,
    };
}

function clone_array_of_pts(xxs: Pt[]): Pt[] {
    var xxs1: Pt[] = [];
    for (var ixs in xxs) {
        xxs1.push([...xxs[ixs]]);
    }
    return xxs1;
}

function clone_universe(u: Universe): Universe {
    return {
        problem: clone_problem(u.problem),
        world: clone_world(u.world),
        history: u.history.map((x) => clone_world(x)),
        invalid: u.invalid.map((x) => clone_invalid(x)),
    };
}

function clone_invalid(i: Invalid): Invalid {
    return {
        violates: i.violates,
        edge: clone_edge(i.edge),
        point: [...i.point]
    };
}

function clone_edge(e: Edge): Edge {
    return {
        from_id: e.from_id,
        to_id: e.to_id,
        from: [...e.from],
        to: [...e.to]
    };
}

function validate_world(world: World, u: Universe): Universe {
    // TODO: Send world to the server and something of Invalid[] back
    var u1 = clone_universe(u);
    var w1 = clone_world(world);
    u1.history.push(w1);
    u1.world = w1;
    return u1;
}
