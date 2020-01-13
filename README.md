# Ripples
My personal desktop background generator.

## Setup
To run Ripples, you'll need the following:
- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) 
- [Node](https://nodejs.org/en/)

## Building
To compile the Rust code to wasm: `wasm-pack build` from the root directory.

To start the frontend dev server: `npm start` from the [www](www) directory.

You can see the application by visiting [localhost:8080](localhost:8080).
Click and drag to see ripples! Press the spacebar to toggle pause.

## Configuring
You'll find a configuration file under `www/params.json`. Here's an explanation of what each field does:
- `freqMin` and `freqMax`: The "frequency" of a ripple is how many frames it takes for a new circle to appear. Each ripple will have a random frequency between `freqMin` and `freqMax`.
- `magMin` and `magMax`: The "magnitude" of a ripple is how large its circle becomes. Each ripple has a random magnitude between `magMin` and `magMax`.
- `backgroundColor`: A color string representing the color of the background of the canvas.
- `autodraw`: Configuration data for automatic ripple generation, explained below.

### Autodraw
Autodraw will draw circles on the canvas, top-down then left-right, according to the configuration parameters. It will then automatically pause.
- `active`: A boolean determining whether to use autodraw.
- `stopFrame`: The number of frames after drawing the final ripple to let the animation run.
- `circlesPerFrame`: The number of new ripples to draw per frame.
- `{x,y}{Start,End}Offs` : The program will attempt to begin drawing ripples beyond the boundaries of the canvas, as specified on each axis by these offsets.
- `{x,y}Step`: The distance between the centers of circles for each coordinate.
- `{x,y}Spread`: The spread parameters introduce an amount of random variance in the coordinate of a ripple along the grid outlined by the steps.

