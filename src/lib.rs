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

type Coordinate = u16;
type DropletStrength = u16;
type Color = u32;
type RippleCtr = u16;

/// `Droplets` contains information about every droplet that has been dropped in the pond.
/// The magnitude of each droplet fades away after a certain amount of time, and a droplet
/// produces ripples at the frequency set by its associated ripple_freq
struct Droplets {
    xs: Vec<Coordinate>,
    ys: Vec<Coordinate>,
    /// The magnitude of the next generated ripple
    next_mags: Vec<DropletStrength>,
    /// The number of iterations that pass before a new ripple is generated
    ripple_freqs: Vec<RippleCtr>,
    /// A counter counting down from ripple_freq to 0, to determine when the next ripple is made
    ripple_ctrs: Vec<RippleCtr>,
    /// The RGB of a color (only the lower 24 bits are used)
    colors: Vec<Color>,
    /// The magnitudes of the children ripples
    /// Though the access pattern on the inner vector would seem to be conducive to a `VecDeque`,
    /// the fact that its contents are exposed to wasm requires coercion to a pointer, hence
    /// necessitating a `Vec`.
    ripple_mags: Vec<Vec<DropletStrength>>,
    ripple_max_mags: Vec<Vec<DropletStrength>>,
    /// The length of each corresponding ripple vec (u32 not usize for wasm)
    ripple_counts: Vec<u32>,
}

const DROPLET_START_CAP: usize = 128;
const RIPPLE_START_CAP: usize = 1024;

impl Droplets {
    pub fn new() -> Droplets {
        Droplets {
            xs: Vec::with_capacity(DROPLET_START_CAP),
            ys: Vec::with_capacity(DROPLET_START_CAP),
            next_mags: Vec::with_capacity(DROPLET_START_CAP),
            ripple_freqs: Vec::with_capacity(DROPLET_START_CAP),
            ripple_ctrs: Vec::with_capacity(DROPLET_START_CAP),
            colors: Vec::with_capacity(DROPLET_START_CAP),
            ripple_mags: Vec::with_capacity(DROPLET_START_CAP),
            ripple_max_mags: Vec::with_capacity(DROPLET_START_CAP),
            ripple_counts: Vec::with_capacity(DROPLET_START_CAP),
        }
    }
}

/// A `Pond` contains all the active droplets and ripples.
#[wasm_bindgen]
pub struct Pond {
    width: Coordinate,
    height: Coordinate,
    droplets: Droplets,
}

#[wasm_bindgen]
impl Pond {
    pub fn new(width: Coordinate, height: Coordinate) -> Pond {
        Pond {
            width,
            height,
            droplets: Droplets::new(),
        }
    }

    /// Updates the pond by generating new ripples, and removing olds ripples
    /// and droplets that have run out of inertia.
    pub fn tick(&mut self) {
        // Add fresh ripples, and remove dead droplets at the same time
        let Droplets {
            xs,
            ys,
            next_mags,
            ripple_freqs,
            ripple_ctrs,
            colors,
            ripple_mags,
            ripple_max_mags,
            ripple_counts,
        } = &mut self.droplets;
        let mut droplet_id = 0;
        while droplet_id != xs.len() {
            // Since we're not using any fancy IDs for droplets, it doesn't matter
            // that we'll visit the same id multiple times (since droplets shift along
            // with the indices)
            let mags = &mut ripple_mags[droplet_id];
            let max_mags = &mut ripple_max_mags[droplet_id];
            let mut new_count = ripple_counts[droplet_id];
            let mut i = 0;
            // Remove or update existing ripples
            while i != mags.len() {
                // Need to increase magnitude by 1 if ripple is not dead
                let mag = mags[i] + 1;
                if mag > max_mags[i] {
                    // Remove inert ripples
                    mags.remove(i);
                    max_mags.remove(i);
                    new_count -= 1;
                } else {
                    mags[i] = mag;
                    i += 1;
                }
            }
            // Update droplet livelihood
            let ripple_ctr = ripple_ctrs[droplet_id];
            let next_new_mag = next_mags[droplet_id];
            if ripple_ctr == 0 {
                // Create new ripple
                mags.push(0);
                max_mags.push(next_new_mag);
                next_mags[droplet_id] = next_new_mag - 1;
                ripple_ctrs[droplet_id] = ripple_freqs[droplet_id];
                ripple_counts[droplet_id] = new_count + 1;
                droplet_id += 1;
            } else if next_new_mag > 1 {
                ripple_counts[droplet_id] = new_count;
                ripple_ctrs[droplet_id] = ripple_ctr - 1;
                droplet_id += 1;
            } else {
                xs.remove(droplet_id);
                ys.remove(droplet_id);
                next_mags.remove(droplet_id);
                ripple_freqs.remove(droplet_id);
                ripple_ctrs.remove(droplet_id);
                colors.remove(droplet_id);
                ripple_mags.remove(droplet_id);
                ripple_max_mags.remove(droplet_id);
                ripple_counts.remove(droplet_id);
            }
        }
    }

    pub fn add_droplet(
        &mut self,
        x: Coordinate,
        y: Coordinate,
        mag: DropletStrength,
        color: Color,
        freq: RippleCtr,
    ) {
        if x >= self.width || y >= self.height || mag == 0 {
            return;
        }
        let Droplets {
            xs,
            ys,
            next_mags,
            ripple_freqs,
            ripple_ctrs,
            colors,
            ripple_mags,
            ripple_max_mags,
            ripple_counts,
        } = &mut self.droplets;
        xs.push(x);
        ys.push(y);
        next_mags.push(mag);
        ripple_freqs.push(freq);
        ripple_ctrs.push(0);
        colors.push(color);
        ripple_mags.push(Vec::with_capacity(RIPPLE_START_CAP));
        ripple_max_mags.push(Vec::with_capacity(RIPPLE_START_CAP));
        ripple_counts.push(0);
    }

    pub fn droplet_count(&self) -> u32 {
        // Seems unlikely that we'll overflow u32 into usize
        self.droplets.xs.len() as u32
    }

    pub fn droplet_xs(&self) -> *const Coordinate {
        self.droplets.xs.as_ptr()
    }

    pub fn droplet_ys(&self) -> *const Coordinate {
        self.droplets.ys.as_ptr()
    }

    pub fn droplet_colors(&self) -> *const Color {
        self.droplets.colors.as_ptr()
    }

    pub fn ripple_mags(&self, droplet_id: usize) -> *const DropletStrength {
        self.droplets.ripple_mags[droplet_id].as_ptr()
    }

    pub fn ripple_max_mags(&self, droplet_id: usize) -> *const DropletStrength {
        self.droplets.ripple_max_mags[droplet_id].as_ptr()
    }

    pub fn ripple_counts(&self) -> *const u32 {
        self.droplets.ripple_counts.as_ptr()
    }
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
}
