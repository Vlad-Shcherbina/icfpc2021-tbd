// MAGIC HAPPENS AROUND LINE 70

import assert from "./assert.js"
import {
    WindowPt, CanvasPt, GridPt,
    Pt, Pair, Figure, Problem, Frame, Foci,
    Actions, CheckPoseRequest, CheckPoseResponse, RotateRequest,
    EdgeStatus,
    ShakeRequest
} from "./types.js"

let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;
let canvas_figure = document.getElementById("figure") as HTMLCanvasElement;
let ctx_figure = canvas_figure.getContext("2d")!;
let canvas_circles = document.getElementById("circles") as HTMLCanvasElement;
let ctx_circles = canvas_circles.getContext("2d")!;
let canvas_foci = document.getElementById("foci") as HTMLCanvasElement;
let ctx_foci = canvas_foci.getContext("2d")!;
let dx = 0;
let dy = 0;

// color scheme
const CLR_HOLE = "#777777";
const CLR_OK_EDGE = "#007F0E";
const CLR_SHORT_EDGE = "#B200FF";
const CLR_LONG_EDGE = "#D10000";
const CLR_SELECTED = "#FFD800";
const CLR_DESELECTED = "#222222";
const CLR_GRID = "#BBBBBB";
const CLR_CIRCLES = "#00FFFF";
const CLR_FOCI = "#A96060";

async function get_problem(n: number): Promise<Problem> {
    const response = await fetch(`/data/problems/${n}.problem`);
    assert(response.ok);
    return await response.json();
}

let problem: Problem;
let figure: Figure;
let frame: Frame;
let foci: Foci = { expected: 0, selected: new Map() };
let selected: boolean[] = [];
let server_check_result: CheckPoseResponse | null = null;

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
    draw_hole();
    draw_grid(frame);
    on_figure_change();

    document.getElementById('problem-stats')!.innerHTML = `
    Problem #${problem_no}: <br>
    ${problem.figure.vertices.length} vertices,
    ${problem.figure.edges.length} edges,
    epsilon = ${problem.epsilon}
    `;

    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.onkeyup = () => {
        // console.log(solution.value);
        figure.vertices = JSON.parse(solution.value!).vertices;
        // console.log(figure.vertices.length, problem.figure.vertices.length);
        assert(figure.vertices.length == problem.figure.vertices.length);
        // console.dir(figure.vertices);
        on_figure_change();
    };

    let circles_checkbox = document.getElementById("show_circles") as HTMLInputElement;
    circles_checkbox.onchange = draw_circles;

    let submit_button = document.getElementById('submit-button') as HTMLButtonElement;
    let submit_result = document.getElementById('submit-result')!;
    submit_button.onclick = async function () {
        submit_result.innerText = 'submitting ...';
        let r = await fetch('/api/submit/' + problem_no, {
            method: 'POST', body: new Blob([solution.value]),
        });
        assert(r.ok);
        submit_result.innerHTML = await r.text();
    };

    document.getElementById('our-submissions')!.innerHTML =
        `<p><a href="https://poses.live/problems/${problem_no}">our submissions</a></p>`;

    (document.getElementById('shake-button') as HTMLButtonElement).onclick = async function () {
        let shake_param = document.getElementById('shake-param') as HTMLInputElement;
        let shake_method = document.getElementById('shake-method') as HTMLSelectElement;
        let req: ShakeRequest = {
            problem: problem,
            vertices: figure.vertices,
            selected,
            method: shake_method.value,
            param: parseInt(shake_param.value),
        };
        let r = await fetch('/api/shake', {
            method: 'POST',
            body: new Blob([JSON.stringify(req)]),
        });
        assert(r.ok);
        figure.vertices = await r.json();
        assert(figure.vertices.length == problem.figure.vertices.length);
        on_figure_change();
    }

    // THIS IS WHERE THE MAGIC HAPPENS

    // Keyboard events
    document.onkeydown = keyboard_handlers;

    // Mouse events
    let mouse_coord: WindowPt | null = null;
    const DRAG_TRIGGER_SENSE_SQUARED = 10 * 10;
    let mouse_dragging: boolean = false;

    // canvas_figure.onmouseup = (e: MouseEvent) => {
    //     if (!e.ctrlKey && !e.shiftKey) selected = figure.vertices.map(_ => false);
    //     select_vertex([e.pageX, e.pageY]);
    // }

    canvas_figure.onmousedown = (e: MouseEvent) => {
        mouse_coord = [e.pageX, e.pageY];
        mouse_dragging = false;
    };

    canvas_figure.onmousemove = (e: MouseEvent) => {
        console.log("caught");
        if (mouse_coord == null) {
            // TODO: show vertex to be selected
            return;
        }
        if (mouse_dragging) {
            drag_vertex(mouse_coord, [e.pageX, e.pageY]);
            mouse_coord = [e.pageX, e.pageY];
            console.log("after", mouse_coord);
            return;
        }
        if (Math.pow(e.pageX - mouse_coord[0], 2) + Math.pow(e.pageY - mouse_coord[0], 2) > DRAG_TRIGGER_SENSE_SQUARED) {
            mouse_dragging = true;
            start_dragging_vertex(mouse_coord);
        }
    };


    canvas_figure.onmouseup = (e: MouseEvent) => {
        if (mouse_coord == null) return;
        if (mouse_dragging) {
            on_figure_change();
            mouse_dragging = false;
        }
        else {
            //console.log("Selecting vertex or focus", foci.expected, foci.selected.size);
            // if (foci.expected > foci.selected.size) {
            //     select_focus([e.pageX, e.pageY]);
            // } else {
            if (!e.ctrlKey && !e.shiftKey) selected = figure.vertices.map(_ => false);
            select_vertex([e.pageX, e.pageY]);
            // }
        }
        mouse_coord = null;
    };

    // Other events, elements and anchors

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

// ===== HANDLERS =====

// Request server for info
function on_figure_change() {
    static_figure_change();
    check_solution_on_server();
}

// Redraw without server requests
function static_figure_change() {
    draw_figure();
    draw_selected();
    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.value = JSON.stringify({ vertices: figure.vertices }, null, 2);
    show_dislikes();
}


async function keyboard_handlers(e: KeyboardEvent) {
    if (e.code == "KeyC" && !e.ctrlKey) {
        e.preventDefault();
        let circles_checkbox = document.getElementById("show_circles") as HTMLInputElement;
        circles_checkbox.checked = !circles_checkbox.checked;
        draw_circles();
        return;
    }

    if ((e.code == "KeyM" || e.code == "KeyN") && !e.ctrlKey) {
        let angle = 0;
        if (e.code == "KeyM") {
            angle = 15;
        } else {
            angle = -15;
        }
        let req: RotateRequest = {
            problem: problem,
            vertices: figure.vertices,
            selected,
            pivot: null,
            angle: angle
        };
        let r = await fetch('/api/rotate', {
            method: 'POST',
            body: new Blob([JSON.stringify(req)]),
        });
        assert(r.ok);
        figure.vertices = await r.json();
        assert(figure.vertices.length == problem.figure.vertices.length);
        on_figure_change();
    }

    let dx = 0;
    let dy = 0;
    if (e.code == "ArrowUp") dy = -1;
    else if (e.code == "ArrowDown") dy = 1;
    else if (e.code == "ArrowLeft") dx = -1;
    else if (e.code == "ArrowRight") dx = +1;
    // else if (e.code == "KeyR") {
    //     if (e.shiftKey) {
    //         foci.expected = 0;
    //         foci.selected = new Map();
    //         return ctx_foci.clearRect(0, 0, canvas_foci.width, canvas_foci.height);
    //     }
    //     console.log("Expecting 1 focus")
    //     foci.expected = 1;
    //     check_for_enough_foci_and_send(Actions.Rotate);
    // }
    else return;
    e.preventDefault();
    move_selected([dx, dy]);
    on_figure_change();
}


// ===== COORD SYSTEMS AND TRANSLATION =====

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

    let width = canvas_hole.width;
    let height = canvas_figure.height;
    dx = width / (frame.max_x - frame.min_x);
    dy = height / (frame.max_y - frame.min_y)

    return frame;
}


function grid_to_canvas(p: GridPt): CanvasPt {
    let nx = Math.floor(dx * (p[0] - frame.min_x)) + 0.5;
    let ny = Math.floor(dy * (p[1] - frame.min_y)) + 0.5;
    return [nx, ny];
}

function canvas_to_grid(p: CanvasPt): GridPt {
    let nx = Math.floor((p[0] - 0.5) / dx + frame.min_x);
    let ny = Math.floor((p[1] - 0.5) / dy + frame.min_y);
    return [nx, ny];
}


function window_to_canvas(mouse_coord: WindowPt): CanvasPt {
    let r = canvas_figure.getBoundingClientRect();
    return [mouse_coord[0] - r.left, mouse_coord[1] - r.top];
}

// ===== REDRAW =====


function draw_grid(frame: Frame) {
    let ctx = ctx_hole;
    ctx.fillStyle = CLR_GRID;
    for (let x = frame.min_x; x < frame.max_x; x++) {
        for (let y = frame.min_y; y < frame.max_y; y++) {
            let p = grid_to_canvas([x, y]);
            ctx.beginPath();
            ctx.arc(p[0], p[1], 1, 0, 2 * Math.PI);
            ctx.fill();
        }
    }
}


function draw_hole() {
    canvas_hole.width = canvas_hole.width;
    let ctx = ctx_hole;
    let hole = problem.hole;
    ctx.strokeStyle = CLR_HOLE;
    ctx.lineWidth = 2;
    ctx.beginPath();
    let p = grid_to_canvas(hole[hole.length - 1])
    ctx.moveTo(...p);
    for (let i = 0; i < hole.length; i++) {
        let p = grid_to_canvas(hole[i]);
        ctx.lineTo(...p);
    }
    ctx.stroke();
}


function draw_figure() {
    canvas_figure.width = canvas_figure.width;
    let ctx = ctx_figure;
    ctx.lineWidth = 2;
    for (let i = 0; i < figure.edges.length; i++) {
        let ok_length = true;
        ctx.setLineDash([]);
        ctx.strokeStyle = CLR_OK_EDGE;
        if (server_check_result != null) {
            let status = server_check_result.edge_statuses[i];
            if (!status.fits_in_hole) {
                ctx.setLineDash([3, 3]);
            }
            if (status.actual_length > status.max_length) {
                ctx.strokeStyle = CLR_LONG_EDGE;
                ok_length = false;
            }
            if (status.actual_length < status.min_length) {
                ctx.strokeStyle = CLR_SHORT_EDGE;
                ok_length = false;
            }
        }

        let [start, end] = figure.edges[i];
        let p1 = grid_to_canvas(figure.vertices[start])
        let p2 = grid_to_canvas(figure.vertices[end])
        ctx.beginPath();
        ctx.moveTo(p1[0], p1[1]);
        ctx.lineTo(p2[0], p2[1]);
        ctx.stroke();
        if (!ok_length) {
            // show limits as text
            assert(server_check_result != null);
            ctx.fillStyle = "#444444";
            let [mx, my] = [(p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2];
            ctx.fillText(`${server_check_result.edge_statuses[i].actual_length
                } (${server_check_result.edge_statuses[i].min_length
                } : ${server_check_result.edge_statuses[i].max_length
                })`, mx, my);
        }
    }
}


function draw_selected() {
    let ctx = ctx_figure;
    for (let i = 0; i < selected.length; i++) {
        ctx.fillStyle = selected[i] ? CLR_SELECTED : CLR_DESELECTED;
        let p = grid_to_canvas(figure.vertices[i]);
        ctx.beginPath();
        ctx.arc(p[0], p[1], 3, 0, 2 * Math.PI);
        ctx.fill();
    }
    draw_circles();
}


function draw_circles() {
    canvas_circles.width = canvas_circles.width
    if (server_check_result == null) return;
    let checkbox = document.getElementById("show_circles") as HTMLInputElement;
    if (!checkbox.checked) return;

    let count = 0;
    let p: number | null = null;
    for (let i = 0; i <= selected.length; ++i) {
        if (!selected[i]) continue;
        if (p != null) return;
        p = i;
    }
    if (p == null) return;

    let edges = []
    for (let i = 0; i < figure.edges.length; ++i) {
        if (figure.edges[i][0] == p || figure.edges[i][1] == p)
            edges.push(i);
    }

    let ctx = ctx_circles;
    ctx.strokeStyle = CLR_CIRCLES;
    for (let i of edges) {
        let start = figure.edges[i][0] == p ? figure.edges[i][1] : figure.edges[i][0];
        let c = grid_to_canvas(figure.vertices[start]);
        let status = server_check_result.edge_statuses[i];
        draw_one_circle(c, Math.sqrt(status.min_length) * dx, Math.sqrt(status.min_length) * dy, CLR_SHORT_EDGE, ctx);
        draw_one_circle(c, Math.sqrt(status.max_length) * dx, Math.sqrt(status.max_length) * dy, CLR_LONG_EDGE, ctx);
    }
}

function draw_one_circle(c: CanvasPt, r1: number, r2: number, color: string, ctx: CanvasRenderingContext2D) {
    ctx.beginPath();
    let prevStyle = ctx.strokeStyle;
    let prevLineWidth = ctx.lineWidth

    ctx.strokeStyle = color;
    ctx.lineWidth = 0.3;
    ctx.ellipse(c[0], c[1], r1, r2, 0, 0, Math.PI * 2);
    ctx.stroke();
    ctx.strokeStyle = prevStyle;
    ctx.lineWidth = prevLineWidth;
}


function draw_foci() {
    console.log("Drawing foci");
    let ctx = ctx_foci;
    const fs0 = ctx.fillStyle;
    ctx.fillStyle = CLR_FOCI;
    foci.selected.forEach((v: Pt, _k: string) => {
        //console.log("Handling", v);
        let p = grid_to_canvas(v);
        //console.log("It becomes", p)
        ctx.rect(p[0] - 3, p[1] - 3, 6, 6);
        ctx.fill();
    })
    ctx.fillStyle = fs0;
}


function show_dislikes() {
    let txt = document.getElementById("score")! as HTMLParagraphElement;
    txt.innerHTML = "Dislikes: ";
    if (server_check_result == null) {
        txt.innerHTML += "waiting...";
        return;
    }
    txt.innerHTML += `${server_check_result.dislikes}`;
    if (!server_check_result.valid) {
        txt.innerHTML += " (not valid)";
    }
}


// ===== INTERACTION =====

let version_counter = 0;
async function check_solution_on_server() {
    let vc = ++version_counter;
    let req: CheckPoseRequest = {
        problem: problem, vertices: figure.vertices
    };
    let r = await fetch('/api/check_pose', {
        method: 'POST', body: new Blob([JSON.stringify(req)]),
    });
    assert(r.ok);
    if (vc != version_counter) return;

    server_check_result = await r.json();
    static_figure_change();
};


// TODO: Return selected vertex_id for clarity of API
function select_vertex(mouse_coord: WindowPt) {
    let i = get_nearby_vertex_index(mouse_coord);
    if (i == null) return;
    selected[i] = !selected[i];
    draw_selected();
}

const VERTEX_CHOOSE_SENSE = 10;
function get_nearby_vertex_index(mouse_coord: WindowPt): number | null {
    let mp = window_to_canvas(mouse_coord);
    for (let i = 0; i < figure.vertices.length; i++) {
        let p = grid_to_canvas(figure.vertices[i]);
        if (Math.pow(p[0] - mp[0], 2) + Math.pow(p[1] - mp[1], 2) < Math.pow(VERTEX_CHOOSE_SENSE, 2)) {
            return i;
        }
    }
    return null;
}

function closest_grid_vertex(mouse_coord: WindowPt): GridPt {
    // for some reason using mouse_to_canvas makes it worse
    // TODO: Check it
    let [x, y] = canvas_to_grid(window_to_canvas(mouse_coord));
    return [Math.floor(x + 0.5), Math.floor(y + 0.5)];
}

function move_selected([dx, dy]: GridPt) {
    for (let i = 0; i < selected.length; i++) {
        if (!selected[i]) continue;
        figure.vertices[i][0] += dx;
        figure.vertices[i][1] += dy;
    }
}

let dragged_vertex: number | any = null;
function start_dragging_vertex(from: WindowPt)  {
    dragged_vertex = get_nearby_vertex_index(from);
}

function drag_vertex(from: WindowPt, to: WindowPt) {
    if (dragged_vertex == null) return;
    let q = closest_grid_vertex(to);
    figure.vertices[dragged_vertex] = q;
    static_figure_change();
}

// TODO: Implement for rotation
function select_focus(mouse_coord: WindowPt) {
    console.log("Selecting focus");
    let p = closest_grid_vertex(mouse_coord);
    let key = p[0] + "," + p[1];
    if (foci.selected.has(key)) {
        foci.selected.delete(key);
    } else {
        foci.selected.set(key, p);
    }
    draw_foci();
}

function check_for_enough_foci_and_send(_action: Actions) {
    if (foci.selected.size !== foci.expected) {
        // TODO: wrong_amount_of_foci_error();
    } else {
        // TODO: query_server_with_action(action);
    }
}

