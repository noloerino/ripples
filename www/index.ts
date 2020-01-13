import seedrandom from "seedrandom";
import randomColor from "randomcolor";
import { Pond, init } from "ripples";
import { memory } from "ripples/ripples_bg";
import params from "./params.json";

init();

const HEIGHT = window.innerHeight;
const WIDTH = window.innerWidth;

// Scales the alpha by this fraction
const GLOBAL_ALPHA_SCALE_NUMER = 1;
const GLOBAL_ALPHA_SCALE_DENOM = 15; 

const pond = Pond.new(WIDTH, HEIGHT);
const canvas = <HTMLCanvasElement> document.getElementById("pond-canvas");
canvas.height = HEIGHT;
canvas.width = WIDTH;

const ctx = canvas.getContext("2d");

var animHandle: number;
var paused = false;
const togglePause = () => {
    if (paused) {
        console.log("Unpaused.");
        animHandle = requestAnimationFrame(interactiveRenderLoop);
    } else {
        console.log("Paused.");
        cancelAnimationFrame(animHandle);
    }
    paused = !paused;
};

const interactiveRenderLoop = () => {
    pond.tick();
    drawPond();
    animHandle = requestAnimationFrame(interactiveRenderLoop);
};

var seed = "aldf";
if (params.backgroundColor) {
  canvas.style.backgroundColor = params.backgroundColor;
  seed = params.backgroundColor;
}

const rng = seedrandom(seed);
let currColor = 0x08FF; // TODO lift state
let currMagnitude = 200;
let currFreq = 50;

const colorIter = function* () {
    while (true) {
        yield parseInt("0x" + randomColor({
            seed: Math.trunc(rng() * 0xFFFFFF),
            hue: "blue",
            luminosity: "bright"
        }).substr(1));
    }
}();

const freqScale = params.freqMax - params.freqMin;
const magScale = params.magMax - params.magMin;
const addDroplet = (x: number, y: number) => {
    currFreq = rng() * freqScale + params.freqMin;
    currMagnitude = rng() * magScale + params.magMin;
    currColor = colorIter.next().value;
    pond.add_droplet(x, y, currMagnitude, currColor, currFreq);
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


// === Interactivity ===

const enableInteractive = () => {
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
    const HOLD_INTERVAL = 5;
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
                togglePause();
        }
    });
};

let renderLoop = interactiveRenderLoop;

// Autodraw
if (params.autodraw.active) {
    console.log("Detected autodraw configuration; interactivity disabled.");
    let {
        stopFrame, // Number of frames to linger after last thing drawn until pause
        circlesPerFrame,
        xStartOffs,
        xEndOffs,
        yStartOffs,
        yEndOffs,
        xSpread,
        ySpread,
        xStep,
        yStep,
    } = params.autodraw;

    // Allow drawing off screen
    // canvas.height += yEndOffs;
    // canvas.width += xEndOffs;

    // After the circles are drawn, wait for some amount of frames
    let waitCountdown = stopFrame;
    const waitRenderLoop = () => {
        pond.tick();
        drawPond();
        if (--waitCountdown == 0) {
            // Pause the animation and begin interactivity
            togglePause();
        } else {
            animHandle = requestAnimationFrame(waitRenderLoop);
        }
    };
    // Allow negative random displacements
    let xShift = xSpread / 2, yShift = ySpread / 2;
    let currX = xStartOffs, currY = yStartOffs;
    // Draw circles somewhat randomly according to the parameters
    const autoRenderLoop = () => {
        // Draw up to circlesPerFrame circles
        for (let i = 0; i < circlesPerFrame; i++) {
            if (currY > HEIGHT + yEndOffs) {
                // Move on to next column
                currY = yStartOffs;
                currX += xStep;
            }
            let offsX = rng() * xSpread - xShift;
            let offsY = rng() * ySpread - yShift;
            addDroplet(currX + offsX, currY + offsY);
            currY += yStep;
        }
        pond.tick();
        drawPond();
        if (currX > WIDTH + xEndOffs) {
            console.log(`Completed autodraw (last coordinate x=${currX}, y=${currY})`);
            // console.log("Reenabling interactivity");
            animHandle = requestAnimationFrame(waitRenderLoop);
            // requestAnimationFrame(interactiveRenderLoop);
        } else {
            animHandle = requestAnimationFrame(autoRenderLoop);
        }
    };
    // Update the global render loop
    renderLoop = autoRenderLoop;
}

enableInteractive();

// === Initialization ===
drawPond();
animHandle = requestAnimationFrame(renderLoop);
