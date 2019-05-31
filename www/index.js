import { Pond, init } from "ripples";
import { memory } from "ripples/ripples_bg"

init();

const HEIGHT = 1000;
const WIDTH = 1000;
const GLOBAL_ALPHA = 0.3

const pond = Pond.new(WIDTH, HEIGHT);
const canvas = document.getElementById("pond-canvas");
canvas.height = HEIGHT;
canvas.width = WIDTH;

const ctx = canvas.getContext("2d");
ctx.globalAlpha = GLOBAL_ALPHA;

const renderLoop = () => {
    pond.tick();
    drawPond();
    requestAnimationFrame(renderLoop);
};

pond.add_droplet(500, 500, 200, 0xFFFFFF, 400);
pond.add_droplet(900, 500, 800, 0x0000FF, 100);

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
        let scaleFactor = 1 - (mags[i] / max_mags[i]);
        let r = Math.trunc((color >> 16) * scaleFactor) & 0xFF;
        let g = Math.trunc((color >> 8) * scaleFactor) & 0xFF;
        let b = Math.trunc((color * scaleFactor)) & 0xFF;
        let colorStr = `rgb(${r.toString(16)},${g.toString(16)},${b.toString(16)})`;
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

drawPond();
requestAnimationFrame(renderLoop);
