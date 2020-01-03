import { Pond, init } from "ripples";
import { memory } from "ripples/ripples_bg";
import seedrandom from "seedrandom";
import randomColor from "randomcolor";

init();

const HEIGHT = window.innerHeight;
const WIDTH = window.innerWidth;

// Scales by 0.6, but reduces floating point ops in a tight loop
const GLOBAL_ALPHA_SCALE_NUMER = 4;//1;
const GLOBAL_ALPHA_SCALE_DENOM = 5;

const pond = Pond.new(WIDTH, HEIGHT);
const canvas = <HTMLCanvasElement> document.getElementById("pond-canvas");
canvas.height = HEIGHT;
canvas.width = WIDTH;

const ctx = canvas.getContext("2d");

const renderLoop = () => {
    pond.tick();
    drawPond();
    requestAnimationFrame(renderLoop);
};

const rng = seedrandom("aldf");
let currColor = 0x08FF; // TODO lift state
let currMagnitude = 200;
let currFreq = 50;

// const hues = ["red", "orange", "blue", "green", "purple", "pink"];
// const hues = ["red", "orange", "yellow"];
const colorIter = function* () {
    while (true) {
        // let hue = hues[Math.trunc(rng() * hues.length)];
        yield parseInt("0x" + randomColor({
            seed: Math.trunc(rng() * 0xFFFFFF),
            // hue: hue,
            luminosity: "bright"
        }).substr(1));
    }
}();

const addDroplet = (x: number, y: number) => {
    // currFreq = rng() * 50 + 10;
    // currMagnitude = rng() * 200 + 150;
    rng(); rng();
    currFreq = 40;
    currMagnitude = 200;
    currColor = colorIter.next().value;
    pond.add_droplet(x, y, currMagnitude, currColor, currFreq);
}

const TAU = 2 * Math.PI;

const drawPond = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    const dropletCount = pond.droplet_count();
    const rippleCount = pond.total_ripples();
    const xs = new Uint16Array(memory.buffer, pond.droplet_xs(), dropletCount);
    const ys = new Uint16Array(memory.buffer, pond.droplet_ys(), dropletCount);
    const colors = new Uint32Array(memory.buffer, pond.droplet_colors(), dropletCount);
    const rippleCounts = new Uint32Array(memory.buffer, pond.ripple_counts(), dropletCount);
    let mags = new Uint16Array(memory.buffer, pond.ripple_mags(), rippleCount);
    let max_mags = new Uint16Array(memory.buffer, pond.ripple_max_mags(), rippleCount);
    let rippleId = 0;
    for (let i = 0; i < dropletCount; i++) {
        let color = colors[i];
        let r = (color >> 16) & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = color & 0xFF;
        let rippleCount = rippleCounts[i];
        let colorStr = `rgb(${r},${g},${b})`;
        ctx.fillStyle = colorStr;
        let nextBound = rippleId + rippleCount;
        for (; rippleId < nextBound; rippleId++) {
            let max_mag = max_mags[rippleId];
            let mag = mags[rippleId];
            // We scale by integers rather than a floating point scalar for efficiency
            let a = ((max_mag - mag) * GLOBAL_ALPHA_SCALE_NUMER) / (max_mag * GLOBAL_ALPHA_SCALE_DENOM);
            ctx.beginPath();
            ctx.globalAlpha = a;
            ctx.arc(
                xs[i],
                ys[i],
                mag * 0.1,
                0,
                TAU,
                false
            );
            ctx.fill();
        }
    }
};

// === Interactivity ===
let mouseDown = false;
let mmCtr = 0;
canvas.addEventListener("mousedown", (e) => {
    mouseDown = e.button === 0;
    mmCtr = 0;
    if (mouseDown) {
        addDroplet(e.offsetX, e.offsetY);
    }
});
// Prevents circles from being drawn too close together when the mouse is held down
const HOLD_INTERVAL = 3; //10;
canvas.addEventListener("mousemove", (e) => {
    if (mouseDown && mmCtr++ % HOLD_INTERVAL === HOLD_INTERVAL - 1) {
        addDroplet(e.offsetX, e.offsetY);
    }
});
canvas.addEventListener("mouseup", (e) => mouseDown = mouseDown && !(e.button === 0));

// Pauses
window.addEventListener("keydown", (e) => {
    switch (e.code) {
        case "Space":
            pond.toggle_pause();
    }
});

// === Initialization === 
drawPond();
requestAnimationFrame(renderLoop);
