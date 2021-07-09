// MAGIC HAPPENS AROUND LINE 70

import assert from "./assert.js"
import { Pt, Pair, Figure, Problem, Frame, Foci, Actions } from "./types.js"

let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let canvas_figure = document.getElementById("figure") as HTMLCanvasElement;
let canvas_f_init = document.getElementById("f_init") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;
let ctx_figure = canvas_figure.getContext("2d")!;
let ctx_f_init = canvas_f_init.getContext("2d")!;


// color scheme
const CLR_HOLE = "#777777";
const CLR_OK_EDGE = "#007F0E";
const CLR_SHORT_EDGE = "#B200FF";
const CLR_LONG_EDGE = "#D10000";
const CLR_SELECTED = "#FFD800";
const CLR_DESELECTED = "#222222";
const CLR_GRID = "#BBBBBB";
const CLR_SHADOW_FIGURE = "#FF9B9B";

let width = canvas_hole.width;
let height = canvas_figure.height;

async function get_problem(n: number): Promise<Problem> {
    const response = await fetch(`/data/problems/${n}.problem`);
    assert(response.ok);
    return await response.json();
}

let problem: Problem;
let figure: Figure;
let frame: Frame;
let foci: Foci;
let selected: boolean[] = [];


async function main() {
    window.addEventListener('hashchange', () => location.reload());
    let problem_no = 1;
    let { hash } = window.location;
    if (hash.startsWith('#')) {
        hash = hash.slice(1);
        problem_no = parseInt(hash);
    }

    problem = await get_problem(problem_no);
    figure = JSON.parse(JSON.stringify(problem.figure));
    selected = problem.figure.vertices.map(_ => false);
    frame = get_frame(problem);
    draw_hole(problem.hole);
    draw_grid(frame);
    // draw_shadow_figure();
    draw_figure();

    document.getElementById('problem-stats')!.innerHTML = `
    Problem #${problem_no}: <br>
    ${problem.figure.vertices.length} vertices,
    ${problem.figure.edges.length} edges,
    epsilon = ${problem.epsilon}
    `;

    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.value = JSON.stringify({ vertices: problem.figure.vertices }, null, 2);
    solution.onkeyup = () => {
        console.log(solution.value);
        figure.vertices = JSON.parse(solution.value!).vertices;
        console.log(figure.vertices.length, problem.figure.vertices.length);
        assert(figure.vertices.length == problem.figure.vertices.length);
        console.dir(figure.vertices);
        draw_figure();
    };

    // THIS IS WHERE THE MAGIC HAPPENS
    document.onmouseup = (e: MouseEvent) => {
        if (!e.ctrlKey) selected = figure.vertices.map(_ => false);
        select_point([e.x, e.y]);
    }
    document.onkeydown = (e: KeyboardEvent) => {
        let dx = 0;
        let dy = 0;
        if (e.code == "ArrowUp") dy = -1;
        else if (e.code == "ArrowDown") dy = 1;
        else if (e.code == "ArrowLeft") dx = -1;
        else if (e.code == "ArrowRight") dx = +1;
        else if (e.code == "r") {
            foci.expected = 1;
            check_for_enough_foci_and_send(Actions.Rotate);
        }
        else return;
        e.preventDefault();
        move_selected([dx, dy]);
        solution.value = JSON.stringify({ vertices: figure.vertices }, null, 2);
    };

    // let MOUSE_COORD: MouseEvent | null = null;
    // const MOUSE_SENSE = 5;
    // let MOUSE_CLICK: boolean = true;

    // document.onmousedown = (e: MouseEvent) => {
    //     MOUSE_COORD = e;
    //     MOUSE_CLICK = true;
    // };

    // document.onmousemove = (e: MouseEvent) => {
    //     if (MOUSE_COORD == null) return;
    //     if (Math.pow(e.x - MOUSE_COORD.x, 2) + Math.pow(e.y - MOUSE_COORD.y, 2) < MOUSE_SENSE) {
    //         MOUSE_CLICK = false;
    //     }
    // };

    // document.onmouseup =  (e: MouseEvent) => {
    //     assert(MOUSE_COORD != null)
    //     if (MOUSE_CLICK) select_point(e, f, frame, 10);
    //     else drag_point(MOUSE_COORD, e, frame, f);
    //     MOUSE_COORD = null;
    // };

    document.getElementById("edge_too_long")!.style.color = CLR_LONG_EDGE;
    document.getElementById("edge_too_short")!.style.color = CLR_SHORT_EDGE;

    (document.getElementById("select_all") as HTMLAnchorElement).onclick = () => {
        selected = figure.vertices.map(_ => true);
        draw_selected();
    };

    (document.getElementById("deselect_all") as HTMLAnchorElement).onclick = () => {
        selected = figure.vertices.map(_ => false);
        draw_selected();
    };
}

main();

function check_for_enough_foci_and_send(_action: Actions) {
    if (foci.selected.size !== foci.expected) {
        // TODO: wrong_amount_of_foci_error();
    } else {
        // TODO: query_server_with_action(action);
    }
}

function get_frame(p: Problem): Frame {
    let frame: Frame = {
        min_x: p.hole[0][0],
        max_x: p.hole[0][0],
        min_y: p.hole[0][1],
        max_y: p.hole[0][1],
    }
    for (let [x, y] of p.hole) {
        if (x < frame.min_x) frame.min_x = x;
        if (x > frame.max_x) frame.max_x = x;
        if (y < frame.min_y) frame.min_y = y;
        if (y > frame.max_y) frame.max_y = y;
    }
    for (let [x, y] of p.figure.vertices) {
        if (x < frame.min_x) frame.min_x = x;
        if (x > frame.max_x) frame.max_x = x;
        if (y < frame.min_y) frame.min_y = y;
        if (y > frame.max_y) frame.max_y = y;
    }
    frame.min_x -= 1;
    frame.min_y -= 1;
    frame.max_x += 2;
    frame.max_y += 2;
    return frame;
}


function grid_to_screen(p: Pt): Pt {
    let nx = Math.floor(width / (frame.max_x - frame.min_x) * (p[0] - frame.min_x)) + 0.5;
    let ny = Math.floor(height / (frame.max_y - frame.min_y) * (p[1] - frame.min_y)) + 0.5;
    return [nx, ny];
}

// function screen_to_grid(p: Pt, frame: Frame): Pt {
//     let nx = Math.floor((frame.max_x - frame.min_x) / width * (p[0] - 0.5) + frame.min_x);
//     let ny = Math.floor((frame.max_y - frame.min_y) / height * (p[1] - 0.5) + frame.min_y);
//     return [nx, ny];
// }

function draw_hole(hole: Pt[]) {
    canvas_hole.width = canvas_hole.width;
    let ctx = ctx_hole;
    ctx.strokeStyle = CLR_HOLE;
    ctx.lineWidth = 2;
    ctx.beginPath();
    let p = grid_to_screen(hole[hole.length - 1])
    ctx.moveTo(...p);
    for (let i = 0; i < hole.length; i++) {
        let p = grid_to_screen(hole[i]);
        ctx.lineTo(...p);
    }
    ctx.stroke();
}

function edge_sq_len([x1, y1]: Pt, [x2, y2]: Pt): number {
    return (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
}


function color_by_edge_len(i: number): string {
    let [start, end] = figure.edges[i];
    let d2 = edge_sq_len(figure.vertices[start], figure.vertices[end]);
    [start, end] = figure.edges[i];
    let d1 = edge_sq_len(problem.figure.vertices[start], problem.figure.vertices[end]);
    if (d2 * 1e6 < (1e6 - problem.epsilon) * d1) {
        return CLR_SHORT_EDGE;
    }
    if (d2 * 1e6 > (1e6 + problem.epsilon) * d1) {
        return CLR_LONG_EDGE;
    }
    return CLR_OK_EDGE;
}

function get_edge_limits(i: number): [number, number, number] {
    let [start, end] = figure.edges[i];
    let d2 = edge_sq_len(figure.vertices[start], figure.vertices[end]);
    [start, end] = figure.edges[i];
    let d1 = edge_sq_len(problem.figure.vertices[start], problem.figure.vertices[end]);
    return [Math.ceil(d1 * (1 - problem.epsilon / 1e6)),
        d2,
    Math.floor(d1 * (1 + problem.epsilon / 1e6))];
}


function draw_edge(v1: Pt, v2: Pt, color: string, ctx: CanvasRenderingContext2D) {
    ctx.strokeStyle = color;
    ctx.beginPath();
    let p1 = grid_to_screen(v1)
    let p2 = grid_to_screen(v2)
    ctx.moveTo(p1[0], p1[1]);
    ctx.lineTo(p2[0], p2[1]);
    ctx.stroke();
}

function draw_figure() {
    canvas_figure.width = canvas_figure.width;
    let ctx = ctx_figure;
    ctx.lineWidth = 2;
    for (let i = 0; i < figure.edges.length; i++) {
        let [start, end] = figure.edges[i];
        let p1 = figure.vertices[start];
        let p2 = figure.vertices[end];
        draw_edge(p1, p2, color_by_edge_len(i), ctx);
        let [min, cur, max] = get_edge_limits(i);
        if (min > cur || max < cur) {
            ctx.fillStyle = "#444444";
            let [mx, my] = grid_to_screen([(p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2]);
            ctx.fillText(`${cur} (${min} : ${max})`, mx, my);
        }
    }
    draw_selected();
    calculate_dislikes();
}

// function draw_shadow_figure() {
//     canvas_f_init.width = canvas_f_init.width;
//     let ctx = ctx_f_init;
//     ctx.lineWidth = 2;
//     for (let i = 0; i < figure.edges.length; i++) {
//         let [start, end] = figure.edges[i];
//         draw_edge(figure.vertices[start],
//                   figure.vertices[end],
//                   color_by_edge_len(i),
//                   ctx);
//     }
//     canvas_f_init.width = canvas_f_init.width;
//     draw_figure(f, ctx_f_init, CLR_SHADOW_FIGURE, 1);
// }

function draw_grid(frame: Frame) {
    let ctx = ctx_hole;
    ctx.fillStyle = CLR_GRID;
    for (let x = frame.min_x; x < frame.max_x; x++) {
        for (let y = frame.min_y; y < frame.max_y; y++) {
            let p = grid_to_screen([x, y]);
            ctx.beginPath();
            ctx.arc(p[0], p[1], 1, 0, 2 * Math.PI);
            ctx.fill();
        }
    }
}

function draw_selected() {
    let ctx = ctx_figure;
    for (let i = 0; i < selected.length; i++) {
        ctx.fillStyle = selected[i] ? CLR_SELECTED : CLR_DESELECTED;
        let p = grid_to_screen(figure.vertices[i]);
        ctx.beginPath();
        ctx.arc(p[0], p[1], 3, 0, 2 * Math.PI);
        ctx.fill();
    }
}


function move_selected([dx, dy]: Pt) {
    for (let i = 0; i < selected.length; i++) {
        if (!selected[i]) continue;
        figure.vertices[i][0] += dx;
        figure.vertices[i][1] += dy;
    }
    draw_figure();
}


function mouse_coords_to_canvas(mouse_coord: Pt): number[] {
    let r = canvas_figure.getBoundingClientRect();
    return [mouse_coord[0] - r.left, mouse_coord[1] - r.top];
}

const VERTEX_CHOOSE_SENSE = 10;

function get_nearby_vertex_index(mouse_coord: Pt): number | null {
    let mp = mouse_coords_to_canvas(mouse_coord);
    for (let i = 0; i < figure.vertices.length; i++) {
        let p = grid_to_screen(figure.vertices[i]);
        if (Math.pow(p[0] - mp[0], 2) + Math.pow(p[1] - mp[1], 2) < Math.pow(VERTEX_CHOOSE_SENSE, 2)) {
            return i;
        }
    }
    return null;
}

let marked: number[] = []

// TODO: Rename to select_vertex_id
// TODO: Return selected vertex_id for clarity of API
function select_point(mouse_coord: Pt) {
    let i = get_nearby_vertex_index(mouse_coord);
    if (i == null) return;
    selected[i] = !selected[i];
    draw_selected();
}

function calculate_dislikes() {
    let sum = 0;
    for (let h of problem.hole) {
        let min = edge_sq_len([frame.min_x, frame.min_y], [frame.max_x, frame.max_y]);
        for (let v of figure.vertices) {
            let d = edge_sq_len(h, v);
            if (d < min) min = d;
        }
        sum += min;
    }
    let txt = document.getElementById("score")! as HTMLParagraphElement;
    txt.innerHTML = `Dislikes: ${sum}`;
}

// TODO: Implement for rotation
function closest_grid_vertex(mouse_coord: Pt): Pt | null {
    return null;
}

// TODO: Implement for rotation
function draw_foci() {
    return { error: "not implemented" };
}

// TODO: Implement for rotation
function select_focus(mouse_coord: Pt) {
    let p = closest_grid_vertex(mouse_coord);
    if (p == null) return;
    let key = p[0] + "," + p[1];
    if (foci.selected.has(key)) {
        foci.selected.delete(key);
    } else {
        foci.selected.set(key, p);
    }
    draw_foci();
}

// function redraw_figure() {
canvas_figure.width = canvas_figure.width;
//     draw_figure(figure, ctx_figure, CLR_OK_EDGE, 2);
//     draw_selected();
// }
