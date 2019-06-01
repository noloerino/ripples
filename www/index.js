import { Pond, init } from "ripples";
import { memory } from "ripples/ripples_bg"

init();

const HEIGHT = 1000;
const WIDTH = 1000;
// const GLOBAL_ALPHA = 0.3
const GLOBAL_ALPHA_SCALE = 0.6;

const pond = Pond.new(WIDTH, HEIGHT);
const canvas = document.getElementById("pond-canvas");
canvas.height = HEIGHT;
canvas.width = WIDTH;

const ctx = canvas.getContext("2d");
// ctx.globalAlpha = GLOBAL_ALPHA;

const renderLoop = () => {
    pond.tick();
    drawPond();
    requestAnimationFrame(renderLoop);
};

let currColor = 0x08FF; // TODO lift state
let currMagnitude = 200;
let currFreq = 50;

let mouseDown = false;
const addDroplet = (e) => {
    if (mouseDown) {
        currColor = Math.trunc(Math.random() * 0xFFFFFF);
        currFreq = Math.random() * 50 + 20;
        currMagnitude = Math.random() * 200 + 100;
        pond.add_droplet(e.offsetX, e.offsetY, currMagnitude, currColor, currFreq);
    }
};

const drawPond = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    const ptrs = [
        pond.ripple_xs(),
        pond.ripple_ys(),
        pond.ripple_mags(),
        pond.ripple_max_mags(),
        pond.ripple_colors(),
    ];
    const rippleCount = pond.ripple_count();
    const [xs, ys, mags, max_mags, colors] = ptrs.map(ptr => new Uint32Array(memory.buffer, ptr, rippleCount));
    for (let i = 0; i < xs.length; i++) {
        let color = (colors[i] & 0xFFFFFF);
        let scaleFactor = (1 - (mags[i] / max_mags[i]));
        let r = Math.trunc((color >> 16)) & 0xFF;
        let g = Math.trunc((color >> 8)) & 0xFF;
        let b = Math.trunc((color)) & 0xFF;
        let a = scaleFactor * GLOBAL_ALPHA_SCALE;
        let colorStr = `rgba(${r},${g},${b},${a})`;
        ctx.fillStyle = colorStr;
        ctx.beginPath();
        ctx.arc(
            xs[i],
            ys[i],
            mags[i],
            0,
            2 * Math.PI,
            false
        );
        ctx.fill();
    }
};

canvas.addEventListener("mousedown", (e) => mouseDown = e.button === 0);
canvas.addEventListener("mousemove", addDroplet);
canvas.addEventListener("mouseup", (e) => mouseDown = mouseDown & !(e.button === 0));

drawPond();
requestAnimationFrame(renderLoop);
