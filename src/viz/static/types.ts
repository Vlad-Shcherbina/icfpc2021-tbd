
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
    TooBig = 1,
    TooSmall,
    EdgeCrossesWall,
}

export interface Invalid {
    violates: Law;
    edge: Pair;
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

export interface Foci {
    expected: number;
    selected: Map<string, Pt>;
}

export enum Actions {
    Rotate = 1,
}
