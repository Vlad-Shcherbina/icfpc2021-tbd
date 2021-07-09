
export type Pt = [number, number];
export type Pair = [number, number];

export interface Figure {
    edges: Pair[];
    vertices: Pt[];
}

export interface Problem {
    hole: Pt[];
    figure: Figure;
    epsilon: number,
}

export interface Frame {
    min_x: number,
    max_x: number,
    min_y: number,
    max_y: number,
}

export type World = Problem;

export enum Law {
    Epsilon = 1,
    EdgeConnectivity,
}

export interface Edge {
    from_id: number;
    to_id: number;
    from: Pt;
    to: Pt;
}

export interface Invalid {
    violates: Law;
    edge: Edge;
    point: Pt;
}

export interface Universe {
    problem: Problem;
    world: World;
    history: World[];
    invalid: Invalid[];
}

export interface Change {
    id: number,
    destination: Pt;
}
