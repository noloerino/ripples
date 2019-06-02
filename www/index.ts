import { Pond, init } from "ripples";
import { memory } from "ripples/ripples_bg";

init();

const HEIGHT = window.innerHeight;
const WIDTH = window.innerWidth;

// Scales by 0.6, but reduces floating point ops in a tight loop
const GLOBAL_ALPHA_SCALE_NUMER = 3;
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

let currColor = 0x08FF; // TODO lift state
let currMagnitude = 200;
let currFreq = 50;

let mouseDown = false;
const addDroplet = (e: MouseEvent) => {
    if (mouseDown) {
        currColor = Math.trunc(Math.random() * 0xFFFFFF);
        currFreq = Math.random() * 50 + 20;
        currMagnitude = Math.random() * 200 + 100;
        pond.add_droplet(e.offsetX, e.offsetY, currMagnitude, currColor, currFreq);
    }
};

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
                mag,
                0,
                TAU,
                false
            );
            ctx.fill();
        }
    }
};

canvas.addEventListener("mousedown", (e) => {
    mouseDown = e.button === 0;
    addDroplet(e);
});
canvas.addEventListener("mousemove", addDroplet);
canvas.addEventListener("mouseup", (e) => mouseDown = mouseDown && !(e.button === 0));

drawPond();
requestAnimationFrame(renderLoop);
