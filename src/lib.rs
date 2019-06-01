mod utils;

use wasm_bindgen::prelude::*;
use web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// From the wasm game of life tutorial:
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

/// A `Droplet` is the source of ripples, i.e. when a click occurs,
/// a new source of ripples should appear and then fade away after
/// a certain amount of time.
/// A `Droplet` is removed when its magnitude reaches 0.
#[derive(Debug)]
#[derive(Clone)]
struct Droplet {
    x: u32,
    y: u32,
    /// The strength of the droplet, i.e. the radius of the first and larget ripple
    mag: u32,
    /// The RGB hex code of this droplet (only the lower 24 bits are used)
    color: u32,
    /// The number of iterations that pass before a new ripple is generated
    ripple_freq: u32,
    /// A counter counting down from ripple_freq to 0, to determine when the next ripple is made
    ripple_ctr: u32,
}

/// A "ripple" is a ripple of water that is created by a droplet.
/// These are what are rendered.
/// A ripple is removed when its radius reaches its maximum radius.
///
/// Here, the "struct of vec" patterns is used for exposure to Javascript. Since
/// `Droplet`s are iterated over for both removal and updating new ripples, they
/// are left as an independent struct.
struct Ripples {
    xs: Vec<u32>,
    ys: Vec<u32>,
    /// The current radius of each ripple
    mags: Vec<u32>,
    /// The maximum radius of a given ripple.
    max_mags: Vec<u32>,
    colors: Vec<u32>,
}

const RIPPLE_START_CAP: usize = 1024;
impl Ripples {
    pub fn new() -> Ripples {
        Ripples {
            xs: Vec::with_capacity(RIPPLE_START_CAP),
            ys: Vec::with_capacity(RIPPLE_START_CAP),
            mags: Vec::with_capacity(RIPPLE_START_CAP),
            max_mags: Vec::with_capacity(RIPPLE_START_CAP),
            colors: Vec::with_capacity(RIPPLE_START_CAP),
        }
    }

    pub fn add_ripple(&mut self, droplet: &Droplet) {
        let &Droplet {
            x,
            y,
            mag,
            color,
            ripple_freq: _,
            ripple_ctr: _,
        } = droplet;
        self.xs.push(x);
        self.ys.push(y);
        self.mags.push(0);
        self.max_mags.push(mag);
        self.colors.push(color);
    }
}

/// A `Pond` contains all the active droplets and ripples.
#[wasm_bindgen]
pub struct Pond {
    width: u32,
    height: u32,
    // No ECS for Droplets for now :(
    droplets: Vec<Droplet>,
    ripples: Ripples,
}

const DROPLET_START_CAP: usize = 128;

#[wasm_bindgen]
impl Pond {
    pub fn new(width: u32, height: u32) -> Pond {
        Pond {
            width,
            height,
            droplets: Vec::with_capacity(DROPLET_START_CAP),
            ripples: Ripples::new(),
        }
    }

    /// Updates the pond by generating new ripples, and removing olds ripples
    /// and droplets that have run out of inertia.
    pub fn tick(&mut self) {
        let Ripples {
            xs,
            ys,
            mags,
            max_mags,
            colors,
        } = &mut self.ripples;
        let mut i = 0;
        while i != mags.len() {
            // Need to increase magnitude by 1 if ripple is not dead
            let mag = mags[i] + 1;
            if mag > max_mags[i] {
                // Remove inert ripples
                xs.remove(i);
                ys.remove(i);
                mags.remove(i);
                max_mags.remove(i);
                colors.remove(i);
            } else {
                mags[i] = mag;
                i += 1;
            }
        }
        // Add fresh ripples, and remove dead droplets at the same time
        let mut new_droplets = Vec::with_capacity(self.droplets.len());
        for droplet in &self.droplets {
            let new_droplet: Droplet;
            let ripple_ctr = droplet.ripple_ctr;
            if ripple_ctr == 0 {
                self.ripples.add_ripple(droplet);
                new_droplet = Droplet {
                    mag: droplet.mag - 1,
                    ripple_ctr: droplet.ripple_freq,
                    ..*droplet
                };
            } else if droplet.mag > 1 {
                new_droplet = Droplet {
                    ripple_ctr: ripple_ctr - 1,
                    ..*droplet
                }
            } else {
                continue;
            }
            new_droplets.push(new_droplet);
        }
        self.droplets = new_droplets;
    }

    pub fn add_droplet(&mut self, x: u32, y: u32, mag: u32, color: u32, freq: u32) {
        if x >= self.width || y >= self.height || mag == 0 {
            return;
        }
        let droplets = &mut self.droplets;
        droplets.push(Droplet {
            x,
            y,
            mag,
            color,
            ripple_freq: freq,
            ripple_ctr: 0,
        });
    }

    pub fn ripple_count(&self) -> usize {
        self.ripples.xs.len()
    }

    /// Returns a pointer to the x coordinates
    pub fn ripple_xs(&self) -> *const u32 {
        self.ripples.xs.as_ptr()
    }

    pub fn ripple_ys(&self) -> *const u32 {
        self.ripples.ys.as_ptr()
    }

    pub fn ripple_mags(&self) -> *const u32 {
        self.ripples.mags.as_ptr()
    }

    pub fn ripple_max_mags(&self) -> *const u32 {
        self.ripples.max_mags.as_ptr()
    }

    pub fn ripple_colors(&self) -> *const u32 {
        self.ripples.colors.as_ptr()
    }
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
}
