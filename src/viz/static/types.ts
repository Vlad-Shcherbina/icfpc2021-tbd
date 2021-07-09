
export type Pt = [number, number];
export type Pair = [number, number];

export interface Figure {
    edges: Pair[];
    vertices: Pt[];
}

export interface Problem {
    hole: Pt[];
    figure: Figure;
}

export interface Frame {
    min_x: number,
    max_x: number,
    min_y: number,
    max_y: number,
}
