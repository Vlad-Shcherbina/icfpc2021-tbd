import assert from "./assert.js"
import {Pt, Pair, Figure, Problem, Frame} from "./types.js"

let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let canvas_figure = document.getElementById("figure") as HTMLCanvasElement;
let canvas_f_init = document.getElementById("f_init") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;
let ctx_figure = canvas_figure.getContext("2d")!;
let ctx_f_init = canvas_f_init.getContext("2d")!;


// color scheme
const CLR_HOLE = "#777777";
const CLR_FIGURE = "#A83F3F";
const CLR_SELECTED = "#0026FF";
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
let frame: Frame;
let selected: boolean[] = [];


async function main() {
    problem = await get_problem(1);
    selected = problem.figure.vertices.map(_ => false);
    frame = get_frame(problem);
    draw_hole(problem.hole, frame);
    draw_grid(frame);
    draw_init_figure(problem.figure, frame);

    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.textContent = JSON.stringify({vertices: problem.figure.vertices}, null, 2);

    document.onmouseup = (e: MouseEvent) => { select_point([e.x, e.y]); }
    document.onkeydown = (e: KeyboardEvent) => {
        let dx = 0;
        let dy = 0;
        if (e.code == "ArrowUp") dy = -1;
        else if (e.code == "ArrowDown") dy = 1;
        else if (e.code == "ArrowLeft") dx = -1;
        else if (e.code == "ArrowRight") dx = +1;
        else return;
        e.preventDefault();
        move_selected([dx, dy]);
        solution.textContent = JSON.stringify({vertices: problem.figure.vertices}, null, 2);
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

    (document.getElementById("select_all") as HTMLAnchorElement).onclick = () => {
        selected = problem.figure.vertices.map(_ => true);
        draw_selected();
    };

    (document.getElementById("deselect_all") as HTMLAnchorElement).onclick = () => {
        selected = problem.figure.vertices.map(_ => false);
        draw_selected();
    };
}

main();


function get_frame(p: Problem) : Frame {
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


function grid_to_screen(p: Pt, frame: Frame): Pt {
    let nx = Math.floor(width / (frame.max_x - frame.min_x) * (p[0] - frame.min_x)) + 0.5;
    let ny = Math.floor(height / (frame.max_y - frame.min_y) * (p[1] - frame.min_y)) + 0.5;
    return [nx, ny];
}

function screen_to_grid(p: Pt, frame: Frame): Pt {
    let nx = Math.floor((frame.max_x - frame.min_x) / width * (p[0] - 0.5) + frame.min_x);
    let ny = Math.floor((frame.max_y - frame.min_y) / height * (p[1] - 0.5) + frame.min_y);
    return [nx, ny];
}

function draw_hole(hole: Pt[], frame: Frame) {
    canvas_hole.width = canvas_hole.width;
    let ctx = ctx_hole;
    ctx.strokeStyle = CLR_HOLE;
    ctx.lineWidth = 2;
    ctx.beginPath();
    let p = grid_to_screen(hole[hole.length - 1], frame)
    ctx.moveTo(...p);
    for (let i = 0; i < hole.length; i++) {
        let p = grid_to_screen(hole[i], frame);
        ctx.lineTo(...p);
    }
    ctx.stroke();
}

function draw_figure(f: Figure, ctx: CanvasRenderingContext2D, color: string, w: number) {
    ctx.strokeStyle = color;
    ctx.lineWidth = w;
    for (let i = 0; i < f.edges.length; i++) {
        ctx.beginPath();
        let p1 = grid_to_screen(f.vertices[f.edges[i][0]], frame)
        let p2 = grid_to_screen(f.vertices[f.edges[i][1]], frame)
        ctx.moveTo(p1[0], p1[1]);
        ctx.lineTo(p2[0], p2[1]);
        ctx.stroke();
    }
}

function draw_init_figure(f: Figure, frame: Frame) {
    canvas_f_init.width = canvas_f_init.width;
    draw_figure(f, ctx_f_init, CLR_SHADOW_FIGURE, 1);
}

function draw_grid(frame: Frame) {
    let ctx = ctx_hole;
    ctx.fillStyle = CLR_GRID;
    for (let x = frame.min_x; x < frame.max_x; x++) {
        for (let y = frame.min_y; y < frame.max_y; y++) {
            let p = grid_to_screen([x, y], frame);
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
        let p = grid_to_screen(problem.figure.vertices[i], frame);
        ctx.beginPath();
        ctx.arc(p[0], p[1], 3, 0, 2 * Math.PI);
        ctx.fill();
    }
}


function move_selected([dx, dy]: Pt) {
    for (let i = 0; i < selected.length; i++) {
        if (!selected[i]) continue;
        problem.figure.vertices[i][0] += dx;
        problem.figure.vertices[i][1] += dy;
    }
    redraw_figure();
}


function mouse_coords_to_canvas(mouse_coord: Pt): number[] {
    let r = canvas_figure.getBoundingClientRect();
    return [mouse_coord[0] - r.left, mouse_coord[1] - r.top];
}

const VERTEX_CHOOSE_SENSE = 10;

function closest(mouse_coord: Pt): number | null {
    let mp = mouse_coords_to_canvas(mouse_coord);
    for (let i = 0; i < problem.figure.vertices.length; i++) {
        let p = grid_to_screen(problem.figure.vertices[i], frame);
        if (Math.pow(p[0] - mp[0], 2) + Math.pow(p[1] - mp[1], 2) < Math.pow(VERTEX_CHOOSE_SENSE, 2)) {
            return i;
        }
    }
    return null;
}

let marked: number[] = []

function select_point(mouse_coord: Pt) {
    let p = closest(mouse_coord);
    if (p == null) return;
    selected[p] = !selected[p];
    draw_selected();
}

function redraw_figure() {
    canvas_figure.width = canvas_figure.width;
    draw_figure(problem.figure, ctx_figure, CLR_FIGURE, 2);
    draw_selected();
}

