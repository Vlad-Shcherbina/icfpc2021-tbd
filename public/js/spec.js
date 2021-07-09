window.spec = {};

window.spec.main = () => {

    window.spec.refresh = new Event('spec_refresh');

    window.spec.state = { "current_problem": {} };

    window.spec.rules = {
        epsilon_factor: 1000000,
    }

    window.spec.state.current_problem = {
        "hole": [
            [55, 80], [65, 95], [95, 95], [35, 5], [5, 5], [35, 50], [5, 95], [35, 95], [45, 80]],
        "figure": {
            "edges": [
                [2, 5], [5, 4], [4, 1], [1, 0], [0, 8], [8, 3], [3, 7], [7, 11], [11, 13], [13, 12], [12, 18], [18, 19], [19, 14], [14, 15], [15, 17], [17, 16], [16, 10], [10, 6], [6, 2], [8, 12], [7, 9], [9, 3], [8, 9], [9, 12], [13, 9], [9, 11], [4, 8], [12, 14], [5, 10], [10, 15]],
            "vertices": [
                [20, 30], [20, 40], [30, 95], [40, 15], [40, 35], [40, 65], [40, 95], [45, 5], [45, 25], [50, 15], [50, 70], [55, 5], [55, 25], [60, 15], [60, 35], [60, 65], [60, 95], [70, 95], [80, 30], [80, 40]
            ]
        }, "epsilon": 150000
    };

    //console.log("Loaded problem, here are vertices:", window.spec.state.current_problem.figure.vertices);

    window.spec.state.current_solution = {
        "hole": [
            [55, 80], [65, 95], [95, 95], [35, 5], [5, 5], [35, 50], [5, 95], [35, 95], [45, 80]],
        "figure": {
            "edges": [
                [2, 5], [5, 4], [4, 1], [1, 0], [0, 8], [8, 3], [3, 7], [7, 11], [11, 13], [13, 12], [12, 18], [18, 19], [19, 14], [14, 15], [15, 17], [17, 16], [16, 10], [10, 6], [6, 2], [8, 12], [7, 9], [9, 3], [8, 9], [9, 12], [13, 9], [9, 11], [4, 8], [12, 14], [5, 10], [10, 15]],
            "vertices": [
                [20, 30], [20, 40], [30, 95], [40, 15], [40, 35], [40, 65], [40, 95], [45, 5], [45, 25], [50, 15], [50, 70], [55, 5], [55, 25], [60, 15], [60, 35], [60, 65], [60, 95], [70, 95], [80, 30], [80, 40]
            ]
        }, "epsilon": 150000
    };

    //console.log("Loaded solution, here are vertices:", window.spec.state.current_solution.figure.vertices);

    // Not used, just an example
    window.spec.example_edge = {
        from_id: 2,
        to_id: 5,
        from_xy: [30, 95],
        to_xy: [40, 65],
    }

    window.spec.d = (p, q) => {
        return Math.pow(p[0] - q[0], 2) + Math.pow(p[1] = q[1], 2);
    }

    window.spec.enforce_epsilon_rule_once = (edge0, edge1, problem = false) => {
        if (!problem) {
            problem = window.spec.state.current_problem;
        }
        if (edge0.from_xy[0] === edge1.from_xy[0] && edge0.from_xy[1] === edge1.from_xy[1]) {
            return true;
        }
        return Math.abs(
            (window.spec.d(edge1.from_xy, edge1.to_xy)) / (window.spec.d(edge0.from_xy, edge0.to_xy)) - 1
        ) <= problem.epsilon / window.spec.rules.epsilon_factor;
    }

    window.spec.enforce_epsilon_rule = (edges0, edges1, problem = false) => {
        if (!problem) {
            problem = window.spec.state.current_problem;
        }
        ////console.log("Checking that second argument:", edges1, "aren't too far from the first argument:", edges0);
        for (x in edges0) {
            //console.log("Checking rules for", edges0[x], "against", edges1[x]);
            if (!window.spec.enforce_epsilon_rule_once(edges0[x], edges1[x], problem)) {
                return { error_at_edge_id: x };
            }
        }
        return edges1.length;
    }

    window.spec.state.history = []

    window.spec.find_edges_by_vertex_id = (vertex_id, figurable) => {
        edges = [...figurable.figure.edges];
        vertices = [...figurable.figure.vertices];
        return edges.reduce((acc, edge) => {
            if (edge[0] === vertex_id || edge[1] === vertex_id) {
                //console.log(edge[0], "x", edge[1]);
                let from_id = edge[0];
                let to_id = edge[1];
                let from_xy = [...vertices[from_id]];
                let to_xy = [...vertices[to_id]];
                //console.log("Relevant edge", edge, "spans", vertices[from_id], "to:", vertices[to_id]);
                let to_push = {
                    from_id: from_id,
                    from_xy: from_xy,
                    to_id: to_id,
                    to_xy: to_xy
                }
                //console.log("Pushing", JSON.stringify(to_push));
                acc.push(to_push);
                return acc;
            } else {
                return acc;
            }
        }, []);
    }

    window.spec.drag = (vertex0_id, vertex1, problem = false, solution = false, history = false) => {
        if (!problem) {
            problem = window.spec.state.current_problem;
        }
        if (!solution) {
            solution = window.spec.state.current_solution;
        }
        if (!history) {
            history = window.spec.state.history;
        }
        //console.log("Dragging", vertex0_id, "which is currently at", solution.figure.vertices[vertex0_id], "to", vertex1);
        rollback = { ...solution };
        //console.log("Updating", vertex0_id, "in solution figure vertex list. Was:", solution.figure.vertices[vertex0_id]);
        history.push(rollback);
        solution.figure.vertices[vertex0_id] = vertex1;
        //console.log("Is", solution.figure.vertices[vertex0_id]);
        //console.log("About to yeet problem", problem);
        edges0 = window.spec.find_edges_by_vertex_id(vertex0_id, problem);
        //console.log("Edges, as specified by the problem", edges0);
        edges1 = window.spec.find_edges_by_vertex_id(vertex0_id, solution);
        //console.log("Edges, after dragging", edges1);
        let enforce_object = window.spec.enforce_epsilon_rule(edges0, edges1, problem);
        //console.log("Enforcing rules", enforce_object, "should be", edges0.length)
        if (edges0.length === enforce_object) {
            window.document.body.dispatchEvent(window.spec.refresh);
            return;
        }
        solution = rollback;
        history.pop();
        return;
    }

}
