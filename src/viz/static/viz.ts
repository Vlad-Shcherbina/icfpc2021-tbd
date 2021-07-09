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
    frame.max_x += 1;
    frame.max_y += 1;
    return frame;
}

function rescale(p: number[], frame: Frame): number[] {
    let nx = Math.floor(width / (frame.max_x - frame.min_x) * (p[0] - frame.min_x));
    let ny = Math.floor(height / (frame.max_y - frame.min_y) * (p[1] - frame.min_y));
    return [nx, ny];
}

function draw_hole(h: hole, frame: Frame) {
    canvas_hole.width = canvas_hole.width;
    // TODO: scale
    let ctx = ctx_hole;
    ctx.strokeStyle = "#777777";
    ctx.lineWidth = 2;
    ctx.beginPath();
    let p = rescale(h.hole[h.hole.length - 1], frame)
    ctx.moveTo(p[0], p[1]);
    for (let i = 0; i < h.hole.length; i++) {
        let p = rescale(h.hole[i], frame);
        ctx.lineTo(p[0], p[1]);
    }
    ctx.stroke();
}

function draw_init_human(f: figure, frame: Frame) {
    canvas_h_init.width = canvas_h_init.width;
    // TODO: scale
    let ctx = ctx_h_init;
    ctx.strokeStyle = "#FF9B9B";
    ctx.lineWidth = 1;
    for (let i = 0; i < f.edges.length; i++) {
        ctx.beginPath();
        let p1 = rescale(f.vertices[f.edges[i][0]], frame)
        let p2 = rescale(f.vertices[f.edges[i][1]], frame)
        ctx.moveTo(p1[0], p1[1]);
        ctx.lineTo(p2[0], p2[1]);
        ctx.stroke();
    }
}

let h: hole = JSON.parse('{ "hole": [[55, 80], [65, 95], [95, 95], [35, 5], [5, 5],[35, 50], [5, 95], [35, 95], [45, 80]] }');
let f: figure = JSON.parse('{"edges": [[2, 5], [5, 4], [4, 1], [1, 0], [0, 8], [8, 3], [3, 7],[7, 11], [11, 13], [13, 12], [12, 18], [18, 19], [19, 14],[14, 15], [15, 17], [17, 16], [16, 10], [10, 6], [6, 2],[8, 12], [7, 9], [9, 3], [8, 9], [9, 12], [13, 9], [9, 11],[4, 8], [12, 14], [5, 10], [10, 15]],"vertices": [[20, 30], [20, 40], [30, 95], [40, 15], [40, 35], [40, 65],[40, 95], [45, 5], [45, 25], [50, 15], [50, 70], [55, 5],[55, 25], [60, 15], [60, 35], [60, 65], [60, 95], [70, 95],[80, 30], [80, 40]]}')
let frame = get_frame(h, f);
draw_hole(h, frame);
draw_init_human(f, frame);

canvas_human.onmousedown = () => { draw_hole(h, frame); };
