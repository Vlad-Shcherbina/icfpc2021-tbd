import assert from "./assert.js"

let canvas_fg = document.getElementById("life_fg") as HTMLCanvasElement;
let ctx_fg = canvas_fg.getContext("2d")!;
let canvas_bg = document.getElementById("life_bg") as HTMLCanvasElement;
let ctx_bg = canvas_fg.getContext("2d")!;
let width = canvas_fg.width;
let height = canvas_fg.height;

let running = false;
let session = 0;
let xcell = 0;
let ycell = 0;

interface ServerLifeResponse {
    session: number;
    field: number[][];
}

async function get_field() {
    assert(session == 0);
    const response = await fetch(`/api/get/`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({session: session})
    });
    const sr: ServerLifeResponse = await response.json();
    session = sr.session;
    draw_bg_field(sr.field);
    draw_field(sr.field);
}

async function get_step() {
    assert(session != 0);
    const response = await fetch(`/api/step/`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({session: session})
    });
    const sr: ServerLifeResponse = await response.json();
    session = sr.session;
    draw_field(sr.field);
    return false;
}

async function change_field(e: MouseEvent) {
    assert(session != 0);
    let r = canvas_fg.getBoundingClientRect();
    let x = Math.floor((e.x - r.left) / xcell);
    let y = Math.floor((e.y - r.top) / ycell);
    const response = await fetch(`/api/change/`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ session: session, x: x, y: y })
    });
    const sr: ServerLifeResponse = await response.json();
    session = sr.session;
    draw_field(sr.field);
}

function draw_bg_field(field: number[][]) {
    xcell = Math.floor((width + 1) / field[0].length);
    ycell = Math.floor((height + 1) / field.length);
    ctx_bg.strokeStyle = '#999999';
    ctx_bg.lineWidth = 1;
    for (let i = 1; i < field[0].length; i++) {
        ctx_bg.beginPath();
        ctx_bg.moveTo(i * xcell + 0.5, 0);
        ctx_bg.lineTo(i * xcell + 0.5, height);
        ctx_bg.stroke();
    }
    for (let i = 1; i < field.length; i++) {
        ctx_bg.beginPath();
        ctx_bg.moveTo(0, i * ycell + 0.5);
        ctx_bg.lineTo(width, i * ycell + 0.5);
        ctx_bg.stroke();
    }
}

function draw_field(field: number[][]) {
    const fwidth = field[0].length;
    const fheight = field.length;
    canvas_fg.width = canvas_fg.width;
    ctx_fg.fillStyle = '#444444';
    for(let i = 0; i < fheight; i++) {
        for (let j = 0; j < fwidth; j++) {
            if (field[i][j] == 0) continue;
            ctx_fg.beginPath();
            ctx_fg.rect(xcell * j + 1, ycell * i + 1, xcell - 1, ycell - 1);
            ctx_fg.fill();
        }
    }
}


async function run_life() {
    if (!running) return;
    await get_step();
    setTimeout(run_life, 70);
}

function run_pause() {
    if (running) running = false;
    else {
        running = true;
        run_life();
    }
}





get_field();
let e = document.getElementById('step')!;
e.onclick = get_step;
let g = document.getElementById('run')!;
g.onmousedown = run_pause;
canvas_fg.onmousedown = change_field;
document.onkeydown = (e: KeyboardEvent) => {
    if (e.code == "ArrowRight") {
        e.preventDefault();
        get_step();
    }
};
// canvas_fg.onmousedown = pressEventHandler;
