// MAGIC HAPPENS AROUND LINE 70

import assert from "./assert.js"
import { Pt, Pair, Figure, Problem, Frame, Foci, 
        Actions, CheckPoseRequest, CheckPoseResponse,
        EdgeStatus } from "./types.js"

let canvas_hole = document.getElementById("hole") as HTMLCanvasElement;
let canvas_figure = document.getElementById("figure") as HTMLCanvasElement;
let ctx_hole = canvas_hole.getContext("2d")!;
let ctx_figure = canvas_figure.getContext("2d")!;
// let canvas_f_init = document.getElementById("f_init") as HTMLCanvasElement;
// let ctx_f_init = canvas_f_init.getContext("2d")!;


// color scheme
const CLR_HOLE = "#777777";
const CLR_OK_EDGE = "#007F0E";
const CLR_SHORT_EDGE = "#B200FF";
const CLR_LONG_EDGE = "#D10000";
const CLR_SELECTED = "#FFD800";
const CLR_DESELECTED = "#222222";
const CLR_GRID = "#BBBBBB";
// const CLR_SHADOW_FIGURE = "#FF9B9B";

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
    draw_hole(problem.hole);
    draw_grid(frame);
    // draw_shadow_figure();
    on_figure_change();

    document.getElementById('problem-stats')!.innerHTML = `
    Problem #${problem_no}: <br>
    ${problem.figure.vertices.length} vertices,
    ${problem.figure.edges.length} edges,
    epsilon = ${problem.epsilon}
    `;

    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.onkeyup = () => {
        console.log(solution.value);
        figure.vertices = JSON.parse(solution.value!).vertices;
        console.log(figure.vertices.length, problem.figure.vertices.length);
        assert(figure.vertices.length == problem.figure.vertices.length);
        console.dir(figure.vertices);
        on_figure_change();
    };

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

    // THIS IS WHERE THE MAGIC HAPPENS

    // Keyboard events
    document.onkeydown = keyboard_handler;

    // Mouse events
    document.onmouseup = (e: MouseEvent) => {
        if (!e.ctrlKey) selected = figure.vertices.map(_ => false);
        select_point([e.x, e.y]);
    }

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

function static_figure_change() {
    draw_figure();
    draw_selected();
    let solution = document.getElementById('solution') as HTMLTextAreaElement;
    solution.value = JSON.stringify({ vertices: figure.vertices }, null, 2);
    let txt = document.getElementById("score")! as HTMLParagraphElement;
    txt.innerHTML = `Dislikes: ${
        server_check_result == null
        ? "waiting..."
        : server_check_result.dislikes
    }`;
}

function on_figure_change() {
    static_figure_change();
    check_solution_on_server();
    // запрос к серверу про состояние
}

function keyboard_handler(e: KeyboardEvent) {
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
    on_figure_change();
}

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
    let width = canvas_hole.width;
    let height = canvas_figure.height;    
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


async function check_solution_on_server() {
    let req: CheckPoseRequest = {
        problem: problem, vertices: figure.vertices
    };
    let r = await fetch('/api/check_pose', {
        method: 'POST', body: new Blob([JSON.stringify(req)]),
    });
    assert(r.ok);

    // TODO: counter
    server_check_result = await r.json();
    static_figure_change();
};


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
        let p1 = grid_to_screen(figure.vertices[start])
        let p2 = grid_to_screen(figure.vertices[end])
        ctx.beginPath();
        ctx.moveTo(p1[0], p1[1]);
        ctx.lineTo(p2[0], p2[1]);
        ctx.stroke();
        if (!ok_length) {
            // show limits as text
            assert(server_check_result != null);
            ctx.fillStyle = "#444444";
            let [mx, my] = [(p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2];
            ctx.fillText(`${
                server_check_result.edge_statuses[i].actual_length
            } (${
                server_check_result.edge_statuses[i].min_length
            } : ${
                server_check_result.edge_statuses[i].max_length
            })`, mx, my);
        }
    }
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

// TODO: Rename to select_vertex_id
// TODO: Return selected vertex_id for clarity of API
function select_point(mouse_coord: Pt) {
    let i = get_nearby_vertex_index(mouse_coord);
    if (i == null) return;
    selected[i] = !selected[i];
    draw_selected();
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

