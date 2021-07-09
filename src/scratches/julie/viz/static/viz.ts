import assert from "./assert.js"

let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let canvas_human = document.getElementById("human") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;
let ctx_human = canvas_human.getContext("2d")!;


let width = canvas_hole.width;
let height = canvas_human.height;

interface hole {
    hole: number[][];
}

interface figure {
    edges: number[][];
    vertices: number[][];
}

function draw_hole(h: hole) {
    // TODO: scale
    let ctx = ctx_hole;
    ctx.strokeStyle = "#777777";
    ctx.beginPath();
    let len = h.hole.length;
    ctx.moveTo(h.hole[len - 1][0], h.hole[len - 1][1]);
    for (let i = 0; i < h.hole.length; i++) {
        ctx.lineTo(h.hole[i][0], h.hole[i][1]);
    }
    ctx.stroke();
}

let h: hole = JSON.parse('{ "hole": [[55, 80], [65, 95], [95, 95], [35, 5], [5, 5],[35, 50], [5, 95], [35, 95], [45, 80]] }');
draw_hole(h);
