// MAGIC HAPPENS AROUND LINE 70

import assert from "./assert.js"
import {
    WindowPt, CanvasPt, GridPt,
    Pt, Pair, Figure, Problem, Frame, Foci, Pose,
    Actions, CheckPoseRequest, CheckPoseResponse, RotateRequest,
    ShakeRequest
} from "./types.js"

// hole, grid, bonus area, draw only on load
let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;

// figure, edge labels and selected vertices, redraw often
let canvas_figure = document.getElementById("figure") as HTMLCanvasElement;
let ctx_figure = canvas_figure.getContext("2d")!;

// circles for goldilocks areas, redraw often
let canvas_circles = document.getElementById("circles") as HTMLCanvasElement;
let ctx_circles = canvas_circles.getContext("2d")!;

// foci
let canvas_foci = document.getElementById("foci") as HTMLCanvasElement;
let ctx_foci = canvas_foci.getContext("2d")!;

// short-lived auxiliary drawing like selection boundary 
let canvas_auxi = document.getElementById("auxi") as HTMLCanvasElement;
let ctx_auxi = canvas_auxi.getContext("2d")!;

// canvas-grid scaling
let dx = 0;
let dy = 0;

// color scheme
const CLR_HOLE = "#777777";
const CLR_OK_EDGE = "#007F0E";
const CLR_SHORT_EDGE = "#B200FF";
const CLR_LONG_EDGE = "#D10000";
const CLR_SELECTED = "#FF6A00";
const CLR_DESELECTED = "#222222";
const CLR_GRID = "#BBBBBB";
const CLR_CIRCLES = "#00FFFF";
const CLR_FOCI = "#A96060";
const CLR_SELECTION_BOUNDARY = "#999999";
const CLR_GLOB_TARGET = "#FFFF00";
const CLR_BREAK_TARGET = "#5555FF";
const CLR_WALL_TARGET = "#FFA500";
const CLR_FLEX_TARGET = "#00FFFF";
const CLR_NEARBY_VERTEX = "#FFD800"

async function get_problem(n: number): Promise<Problem> {
    const response = await fetch(`/data/problems/${n}.problem`);
    assert(response.ok);
    return await response.json();
}

let problem: Problem;
// let figure: Figure;
let highscore: string;
let pose: Pose;
let history: Pose[] = [];
let frame: Frame;
let foci: Foci = { expected: 0, selected: new Map() };
let selected: boolean[] = [];
let server_check_result: CheckPoseResponse;

async function show_problem_stats(problem_no: number) {
    let hs = await fetch('/api/highscore/' + problem_no);
    assert(hs.ok);
    highscore = await hs.text();

    let problem_stats = document.getElementById('problem-stats')!;
    problem_stats.innerHTML = `
    Problem #${problem_no}: <br>
    ${problem.figure.vertices.length} vertices,
    ${problem.figure.edges.length} edges,
    epsilon = ${problem.epsilon},
    bonuses = |`;
    for (let b of problem.bonuses) {
        problem_stats.innerHTML += ` ${b.bonus} for ${b.problem} |`;
    }
    problem_stats.innerHTML += "] <br>";
    problem_stats.innerHTML += highscore;
}

async function main() {
    ctx_figure.fillText("If you can read it, check_pose runs too slow", 10, 10);

    window.addEventListener('hashchange', () => location.reload());
    let problem_no = 1;
    let { hash } = window.location;
    if (hash.startsWith('#')) {
        hash = hash.slice(1);
        problem_no = parseInt(hash);
    }

    problem = await get_problem(problem_no);
    show_problem_stats(problem_no);
    pose = { vertices: JSON.parse(JSON.stringify(problem.figure.vertices)),
              bonuses: [] };
    selected = problem.figure.vertices.map(_ => false);
    frame = get_frame(problem);
    canvas_hole.width = canvas_hole.width;
    draw_grid();
    draw_hole();
    await check_solution_on_server();
    on_figure_change();


    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.onblur = () => {
        // console.log(solution.value);
        pose.vertices = JSON.parse(solution.value!).vertices;
        // console.log(figure.vertices.length, problem.figure.vertices.length);
        assert(pose.vertices.length == problem.figure.vertices.length);
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
        setTimeout(() => {
            show_problem_stats(problem_no);
        }, 2000);
    };

    document.getElementById('our-submissions')!.innerHTML =
        `<p><a href="https://poses.live/problems/${problem_no}">our submissions</a></p>`;

    let shakers: string[] = [
        "random",
        "banana",
        "ice",
        "mango",
        "greedy",
        "springs",
        "threshold",
        "daiquiri",
        "mojito",
    ];
    let shakerdiv = document.getElementById('shakers') as HTMLDivElement;
    for (let method of shakers) {
        shakerdiv.innerHTML += `<button id="${method}">${method}</button> `;
    };
    for (let method of shakers) {
        (document.getElementById(method) as HTMLInputElement).onclick = async function () {
            for (let b of shakerdiv.childNodes) (b as HTMLInputElement).disabled = true;
            let shake_param = document.getElementById('shake-param') as HTMLInputElement;
            let req: ShakeRequest = {
                problem: problem,
                vertices: pose.vertices,
                selected,
                method,
                param: parseInt(shake_param.value),
            };
            let r = await fetch('/api/shake', {
                method: 'POST',
                body: new Blob([JSON.stringify(req)]),
            });
            assert(r.ok);
            pose.vertices = await r.json();
            assert(pose.vertices.length == problem.figure.vertices.length);
            for (let b of shakerdiv.childNodes) (b as HTMLInputElement).disabled = false;
            on_figure_change();           
        }
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
    //     select_vertex([e.x, e.y]);
    // }

    canvas_figure.onmousedown = (e: MouseEvent) => {
        mouse_coord = [e.x, e.y];
        mouse_dragging = false;
    };

    let last_nearby: number | null = null;
    canvas_figure.onmousemove = (e: MouseEvent) => {
        if (mouse_coord == null) {
            // mouse is not down, show the closest vertex
            if (last_nearby != null) {
                draw_one_vertex(last_nearby, selected[last_nearby] ? CLR_SELECTED : CLR_DESELECTED);
            }
            last_nearby = get_nearby_vertex_index([e.x, e.y]);
            if (last_nearby != null) draw_one_vertex(last_nearby, CLR_NEARBY_VERTEX);
            return;
        }
        if (mouse_dragging) {
            // mouse is down and is being drugged
            if (dragged_vertex == null) {
                // started from vertex - drag it somewhere else
                if (!e.ctrlKey && !e.shiftKey) selected = pose.vertices.map(_ => false);
                select_range(mouse_coord, [e.x, e.y]);
            }
            else {
                // started from no vertex - selection bound
                drag_vertex([e.x, e.y]);
                mouse_coord = [e.x, e.y];
            }
            return;
        }
        if (Math.pow(e.x - mouse_coord[0], 2) + Math.pow(e.y - mouse_coord[1], 2) > DRAG_TRIGGER_SENSE_SQUARED) {
            // it's now officially dragging, not click
            mouse_dragging = true;
            start_dragging_vertex(mouse_coord);
        }
    };


    canvas_figure.onmouseup = (e: MouseEvent) => {
        if (mouse_coord == null) return;
        if (mouse_dragging) {
            canvas_auxi.width = canvas_auxi.width;
            on_figure_change();
        }
        else {
            //console.log("Selecting vertex or focus", foci.expected, foci.selected.size);
            // if (foci.expected > foci.selected.size) {
            //     select_focus([e.x, e.y]);
            // } else {
            if (!e.ctrlKey && !e.shiftKey) selected = pose.vertices.map(_ => false);
            select_vertex([e.x, e.y]);
            // }
        }
        mouse_coord = null;
    };

    // Other events, elements and anchors

    document.getElementById("edge_too_long")!.style.color = CLR_LONG_EDGE;
    document.getElementById("edge_too_short")!.style.color = CLR_SHORT_EDGE;
}

main();

// ===== HANDLERS =====

// Request server for info
function on_figure_change() {
    history.push(JSON.parse(JSON.stringify(pose)));
    check_solution_on_server();
    static_figure_change();
}

// Redraw without server requests
function static_figure_change() {
    draw_figure();
    draw_selected();
    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.value = JSON.stringify(pose, null, 2);
    show_dislikes_and_bonuses();
}


async function keyboard_handlers(e: KeyboardEvent) {
    // if (e.ctrlKey || e.metaKey) return;
    if (document.activeElement?.tagName.toLowerCase() == "textarea") return;
    if (e.code == "KeyA") {
        e.preventDefault();
        selected = pose.vertices.map(_ => true);
        draw_selected();
        return;
    }
    if (e.code == "Escape") {
        e.preventDefault();
        selected = pose.vertices.map(_ => false);
        draw_selected();
        return;
    }
    if (e.code == "KeyC") {
        e.preventDefault();
        let circles_checkbox = document.getElementById("show_circles") as HTMLInputElement;
        circles_checkbox.checked = !circles_checkbox.checked;
        draw_circles();
        return;
    }
    if (e.code == "KeyZ") {
        e.preventDefault();
        undo();
        return;
    }

    if (e.code == "KeyM" || e.code == "KeyN") {
        let angle = 0;
        if (e.code == "KeyM") {
            angle = e.shiftKey ? 90 : 15;
        } else {
            angle = e.shiftKey ? -90 : -15;
        }
        e.preventDefault();
        await turn(angle);
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
    if (e.shiftKey) { dx *= 10; dy *= 10; }
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
    let nx = Math.floor((p[0] - 0.5) / dx + frame.min_x + 0.5);
    let ny = Math.floor((p[1] - 0.5) / dy + frame.min_y + 0.5);
    return [nx, ny];
}


function window_to_canvas(mouse_coord: WindowPt): CanvasPt {
    let r = canvas_figure.getBoundingClientRect();
    return [mouse_coord[0] - r.left, mouse_coord[1] - r.top];
}

// ===== REDRAW =====

function draw_grid() {
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
    let ctx = ctx_hole;

    for (let p of problem.bonuses) {
        let [px, py] = grid_to_canvas(p.position);
        const r = 10;
        console.log(p);
        if (p.bonus == "GLOBALIST") {
            ctx.fillStyle = CLR_GLOB_TARGET;
        }
        else if (p.bonus == "BREAK_A_LEG") {
            ctx.fillStyle = CLR_BREAK_TARGET;
        }
        else if (p.bonus == "WALLHACK") {
            ctx.fillStyle = CLR_WALL_TARGET;
        }
        else if (p.bonus == "SUPERFLEX") {
            ctx.fillStyle = CLR_FLEX_TARGET;
        }
        else {
            assert(false, p.bonus);
        }
        ctx.beginPath();
        ctx.arc(px, py, r, 0, 2 * Math.PI);
        ctx.fill();
    }

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
    let edges = derive_edged_from_pose();
    for (let i = 0; i < edges.length; i++) {
        let ok_length = true;
        ctx.setLineDash([]);
        ctx.strokeStyle = CLR_OK_EDGE;
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

        let [start, end] = edges[i];
        let p1 = grid_to_canvas(pose.vertices[start])
        let p2 = grid_to_canvas(pose.vertices[end])
        ctx.beginPath();
        ctx.moveTo(p1[0], p1[1]);
        ctx.lineTo(p2[0], p2[1]);
        ctx.stroke();
        if (!ok_length) {
            // show limits as text
            ctx.fillStyle = "#444444";
            let [mx, my] = [(p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2];
            ctx.fillText(`${status.actual_length} (${status.min_length} : ${status.max_length})`, mx, my);
        }
    }
}


function draw_selected() {
    for (let i = 0; i < selected.length; i++) {
        draw_one_vertex(i, selected[i] ? CLR_SELECTED : CLR_DESELECTED);
    }
    draw_circles();
}

function draw_one_vertex(i: number, clr: string) {
    ctx_figure.fillStyle = clr;
    let p = grid_to_canvas(pose.vertices[i]);
    ctx_figure.beginPath();
    ctx_figure.arc(p[0], p[1], 3, 0, 2 * Math.PI);
    ctx_figure.fill();
}

function draw_circles() {
    canvas_circles.width = canvas_circles.width;
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

    let adj_edges = [];
    let edges = derive_edged_from_pose();
    for (let i = 0; i < edges.length; ++i) {
        if (edges[i][0] == p || edges[i][1] == p)
            adj_edges.push(i);
    }

    let ctx = ctx_circles;
    ctx.strokeStyle = CLR_CIRCLES;
    for (let i of adj_edges) {
        let start = edges[i][0] == p ? edges[i][1] : edges[i][0];
        let c = grid_to_canvas(pose.vertices[start]);
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


function show_dislikes_and_bonuses() {
    let txt = document.getElementById("score")! as HTMLParagraphElement;
    txt.innerHTML = "Dislikes: ";
    txt.innerHTML += `${server_check_result.dislikes} `;
    let extra = false;
    if (!server_check_result.valid) {
        txt.innerHTML += " (not valid";
        extra = true;
    }
    for (let i = 0; i < problem.bonuses.length; ++i) {
        if (server_check_result.unlocked[i]) {
            txt.innerHTML += extra ? ", " : "(";
            txt.innerHTML += `${problem.bonuses[i].problem} unlocked`;
            extra = true;
        }
    }
    if (extra) txt.innerHTML += ")";
}


// ===== INTERACTION =====

let version_counter = 0;
async function check_solution_on_server() {
    let vc = ++version_counter;
    let rpose = { vertices: pose.vertices, bonuses: [] /* TODO*/ }
    let req: CheckPoseRequest = { problem, pose: rpose };
    let r = await fetch('/api/check_pose', {
        method: 'POST', body: new Blob([JSON.stringify(req)]),
    });
    assert(r.ok);
    if (vc != version_counter) return;

    server_check_result = await r.json();
    static_figure_change();
};


function select_vertex(mouse_coord: WindowPt) {
    let i = get_nearby_vertex_index(mouse_coord);
    if (i == null) return;
    selected[i] = !selected[i];
    draw_selected();
    return i;
}


function select_range(from: WindowPt, to: WindowPt) {
    let [fpx, fpy] = window_to_canvas(from);
    let [tpx, tpy] = window_to_canvas(to)
    let [fx, fy] = canvas_to_grid([fpx, fpy]);
    let [tx, ty] = canvas_to_grid([tpx, tpy]);
    let [x1, y1] = [Math.min(tx, fx), Math.min(ty, fy)];
    let [x2, y2] = [Math.max(tx, fx), Math.max(ty, fy)];
    for (let i = 0; i < pose.vertices.length; ++i) {
        let v = pose.vertices[i];
        if (v[0] >= x1 && v[0] <= x2 && v[1] >= y1 && v[1] <= y2) {
            selected[i] = true;
        }
    }

    canvas_auxi.width = canvas_auxi.width;
    ctx_auxi.strokeStyle = CLR_SELECTION_BOUNDARY;
    ctx_auxi.setLineDash([1, 1]);
    ctx_auxi.lineWidth = 2;
    ctx_auxi.beginPath();
    ctx_auxi.rect(fpx, fpy, tpx - fpx, tpy - fpy);
    ctx_auxi.stroke();
    draw_selected();
}

const VERTEX_CHOOSE_SENSE = 10;
function get_nearby_vertex_index(mouse_coord: WindowPt): number | null {
    let mp = window_to_canvas(mouse_coord);
    for (let i = 0; i < pose.vertices.length; i++) {
        let p = grid_to_canvas(pose.vertices[i]);
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
        pose.vertices[i][0] += dx;
        pose.vertices[i][1] += dy;
    }
}

let dragged_vertex: number | any = null;
function start_dragging_vertex(from: WindowPt)  {
    dragged_vertex = get_nearby_vertex_index(from);
}

function drag_vertex(to: WindowPt) {
    if (dragged_vertex == null) return;
    let q = closest_grid_vertex(to);
    pose.vertices[dragged_vertex] = q;
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

function derive_edged_from_pose(): Pair[] {
    // TODO : break a leg
    return problem.figure.edges;
}

function undo() {
    if (history.length < 2) return;
    history.pop();
    pose = history.pop()!;
    on_figure_change();
}

async function turn(angle: number) {
    let req: RotateRequest = {
        problem: problem,
        vertices: pose.vertices,
        selected,
        pivot: null,
        angle: angle
    };
    let r = await fetch('/api/rotate', {
        method: 'POST',
        body: new Blob([JSON.stringify(req)]),
    });
    assert(r.ok);
    pose.vertices = await r.json();
    assert(pose.vertices.length == problem.figure.vertices.length);
    on_figure_change();
}
