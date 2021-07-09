import assert from "./assert.js"

let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let canvas_human = document.getElementById("human") as HTMLCanvasElement;
let canvas_h_init = document.getElementById("h_init") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;
let ctx_human = canvas_human.getContext("2d")!;
let ctx_h_init = canvas_h_init.getContext("2d")!;


let width = canvas_hole.width;
let height = canvas_human.height;

interface hole {
    hole: number[][];
}

interface figure {
    edges: number[][];
    vertices: number[][];
}

interface Frame {
    min_x: number,
    max_x: number,
    min_y: number,
    max_y: number,
}

function get_frame(h: hole, f: figure) : Frame {
    let frame: Frame = {
        min_x: h.hole[0][0],
        max_x: h.hole[0][0],
        min_y: h.hole[0][0],
        max_y: h.hole[0][0],
    }
    for (let p of h.hole) {
        if (p[0] < frame.min_x) frame.min_x = p[0];
        if (p[0] > frame.max_x) frame.max_x = p[0];
        if (p[1] < frame.min_y) frame.min_y = p[1];
        if (p[1] > frame.max_y) frame.max_y = p[1];
    }
    for (let p of f.vertices) {
        if (p[0] < frame.min_x) frame.min_x = p[0];
        if (p[0] > frame.max_x) frame.max_x = p[0];
        if (p[1] < frame.min_y) frame.min_y = p[1];
        if (p[1] > frame.max_y) frame.max_y = p[1];
    }
    frame.min_x -= 1;
    frame.min_y -= 1;
    frame.max_x += 2;
    frame.max_y += 2;
    return frame;
}

function grid_to_screen(p: number[], frame: Frame): number[] {
    let nx = Math.floor(width / (frame.max_x - frame.min_x) * (p[0] - frame.min_x)) + 0.5;
    let ny = Math.floor(height / (frame.max_y - frame.min_y) * (p[1] - frame.min_y)) + 0.5;
    return [nx, ny];
}

function screen_to_grid(p: number[], frame: Frame): number[] {
    let nx = Math.floor((frame.max_x - frame.min_x) / width * (p[0] - 0.5) + frame.min_x);
    let ny = Math.floor((frame.max_y - frame.min_y) / height * (p[1] - 0.5) + frame.min_y);
    return [nx, ny];
}

function draw_hole(h: hole, frame: Frame) {
    canvas_hole.width = canvas_hole.width;
    // TODO: scale
    let ctx = ctx_hole;
    ctx.strokeStyle = "#777777";
    ctx.lineWidth = 2;
    ctx.beginPath();
    let p = grid_to_screen(h.hole[h.hole.length - 1], frame)
    ctx.moveTo(p[0], p[1]);
    for (let i = 0; i < h.hole.length; i++) {
        let p = grid_to_screen(h.hole[i], frame);
        ctx.lineTo(p[0], p[1]);
    }
    ctx.stroke();
}

function draw_human(f: figure, frame: Frame, ctx: CanvasRenderingContext2D, color: string, w: number) {
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

function draw_init_human(f: figure, frame: Frame) {
    canvas_h_init.width = canvas_h_init.width;
    draw_human(f, frame, ctx_h_init, "#FF9B9B", 1);
}

function draw_grid(frame: Frame) {
    let ctx = ctx_hole;
    ctx.fillStyle = "#BBBBBB";
    for (let x = frame.min_x; x < frame.max_x; x++) {
        for (let y = frame.min_y; y < frame.max_y; y++) {
            let p = grid_to_screen([x, y], frame);
            ctx.beginPath();
            ctx.arc(p[0], p[1], 1, 0, 2 * Math.PI);
            ctx.fill();
        }
    }
}

function mouse_coords_to_canvas(e: MouseEvent): number[] {
    let r = canvas_human.getBoundingClientRect();
    return [e.x - r.left, e.y - r.top];
}

function closest(e: MouseEvent, f: figure, frame: Frame, delta: number): number | null {
    let mp = mouse_coords_to_canvas(e);
    for (let i = 0; i < f.vertices.length; i++) {
        let p = grid_to_screen(f.vertices[i], frame);
        if ((p[0] - mp[0]) * (p[0] - mp[0]) + (p[1] - mp[1]) * (p[1] - mp[1]) < delta * delta) {
            return i;
        }
    }
    return null;
}

let marked: number[] = []

function mark_point(e: MouseEvent, f: figure, frame: Frame, delta: number) {
    let p = closest(e, f, frame, delta);
    if (p == null) return;
    let i = marked.indexOf(p);
    if (i == -1) {
        marked.push(p);
    }
    else {
        marked.splice(i, 1);
    }
    redraw_figure(f, frame, marked);
}

function draw_marked_points(m: number[], f: figure, frame: Frame) {
    let ctx = ctx_human;
    ctx.fillStyle = "#0026FF";
    for (let m of marked) {
        let p = grid_to_screen(f.vertices[m], frame);
        ctx.beginPath();
        ctx.arc(p[0], p[1], 4, 0, 2 * Math.PI);
        ctx.fill();
    }
}

function redraw_figure(f: figure, frame: Frame, marked: number[]) {
    canvas_human.width = canvas_human.width;
    draw_human(f, frame, ctx_human, "#A83F3F", 2);
    draw_marked_points(marked, f, frame);
}

let h: hole = JSON.parse('{ "hole": [[55, 80], [65, 95], [95, 95], [35, 5], [5, 5],[35, 50], [5, 95], [35, 95], [45, 80]] }');
let f: figure = JSON.parse('{"edges": [[2, 5], [5, 4], [4, 1], [1, 0], [0, 8], [8, 3], [3, 7],[7, 11], [11, 13], [13, 12], [12, 18], [18, 19], [19, 14],[14, 15], [15, 17], [17, 16], [16, 10], [10, 6], [6, 2],[8, 12], [7, 9], [9, 3], [8, 9], [9, 12], [13, 9], [9, 11],[4, 8], [12, 14], [5, 10], [10, 15]],"vertices": [[20, 30], [20, 40], [30, 95], [40, 15], [40, 35], [40, 65],[40, 95], [45, 5], [45, 25], [50, 15], [50, 70], [55, 5],[55, 25], [60, 15], [60, 35], [60, 65], [60, 95], [70, 95],[80, 30], [80, 40]]}')
let frame = get_frame(h, f);
draw_hole(h, frame);
draw_grid(frame);
draw_init_human(f, frame);

canvas_human.onmousedown = (e: MouseEvent) => { mark_point(e, f, frame, 10); };
