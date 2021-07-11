
export type Pt = [number, number];
export type Pair = [number, number];

export type WindowPt = Pt;
export type CanvasPt = Pt;
export type GridPt = Pt;

export interface Figure {
    edges: Pair[];
    vertices: GridPt[];
}

export interface Problem {
    hole: GridPt[];
    figure: Figure;
    epsilon: number,
    bonuses: ProblemBonus[]
}

export interface Pose {
    vertices: Pt[],
    bonuses: PoseBonus[],
}

export interface Frame {
    min_x: number,
    max_x: number,
    min_y: number,
    max_y: number,
}

export interface EdgeStatus {
    fits_in_hole: boolean,
    actual_length: number,
    min_length: number,
    max_length: number,
}

export interface ProblemTgtBonus {
    bonus: string,
    from_problem: number,
}

export interface ProblemBonus {
    bonus: string,
    problem: number,
    position: GridPt,
}

export interface PoseBonus {
    bonus: string,
    problem: number,
    edge: Pair[] | null,
}

export interface CheckPoseRequest {
    problem: Problem,
    pose: Pose,
}

export interface CheckPoseResponse {
    edges: Pair[],
    edge_statuses: EdgeStatus[],
    dislikes: number,
    valid: boolean,
    unlocked: boolean[],
    bonus_globalist_sum: number | null,
}

export interface ShakeRequest {
    problem: Problem,
    vertices: GridPt[],
    selected: boolean[],
    method: string,
    param: number,
}

export interface RotateRequest {
    problem: Problem,
    vertices: Pt[],
    selected: boolean[],
    pivot: Pt | null,
    angle: number,
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
    destination: GridPt;
}

export interface Foci {
    expected: number;
    selected: Map<string, GridPt>;
}

export enum Actions {
    Rotate = 1,
}
