mod utils;

use wasm_bindgen::prelude::*;

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
    /// The magnitudes of the children ripples, acting as a flattened 2d array
    /// To add a ripple from the `i`th droplet, we must first sum up the number of ripples of
    /// the first `i-1` droplets and add the number of ripples the `i`th droplet currently has;
    /// this computation is a tradeoff (since it happens relatively infrequently) in order to
    /// allow passing a single flat Uint16Array to JS.
    ripple_mags: Vec<DropletStrength>,
    ripple_max_mags: Vec<DropletStrength>,
    /// The length of each corresponding ripple vec (u32 not usize for wasm)
    ripple_counts: Vec<u32>,
    total_ripples: u32,
}

const DROPLET_START_CAP: usize = 128;

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
            total_ripples: 0,
        }
    }
}

/// A `Pond` contains all the active droplets and ripples.
#[wasm_bindgen]
pub struct Pond {
    width: Coordinate,
    height: Coordinate,
    droplets: Droplets,
    paused: bool,
}

#[wasm_bindgen]
impl Pond {
    pub fn new(width: Coordinate, height: Coordinate) -> Pond {
        Pond {
            width,
            height,
            droplets: Droplets::new(),
            paused: false,
        }
    }

    /// Updates the pond by generating new ripples, and removing olds ripples
    /// and droplets that have run out of inertia.
    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
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
            total_ripples,
        } = &mut self.droplets;
        let mut droplet_id = 0;
        let old_total_ripples = *total_ripples;
        let mut new_total_ripples = 0;
        let mut ripple_id = 0;
        let mut new_ripple_mags = Vec::with_capacity(old_total_ripples as usize);
        let mut new_ripple_max_mags = Vec::with_capacity(old_total_ripples as usize);
        while droplet_id != xs.len() {
            // Since we're not using any fancy IDs for droplets, it doesn't matter
            // that we'll visit the same id multiple times (since droplets shift along
            // with the indices)
            let mut new_count = ripple_counts[droplet_id];
            // Remove or update existing ripples
            let ripple_bound = ripple_id + new_count;
            while ripple_id < ripple_bound {
                // Need to increase magnitude by 1 if ripple is not dead
                let mag = ripple_mags[ripple_id as usize] + 1;
                let max_mag = ripple_max_mags[ripple_id as usize];
                if mag > max_mag {
                    // Remove inert ripples
                    new_count -= 1;
                } else {
                    // Preserve for next tick
                    new_ripple_mags.push(mag);
                    new_ripple_max_mags.push(max_mag);
                }
                ripple_id += 1;
            }
            // Update droplet livelihood
            let ripple_ctr = ripple_ctrs[droplet_id];
            let next_new_mag = next_mags[droplet_id];
            if ripple_ctr == 0 {
                // Create new ripple
                new_ripple_mags.push(0);
                new_ripple_max_mags.push(next_new_mag);
                new_count += 1;
                ripple_counts[droplet_id] = new_count;
                // Update droplet
                next_mags[droplet_id] = next_new_mag - 1;
                ripple_ctrs[droplet_id] = ripple_freqs[droplet_id];
                droplet_id += 1;
                new_total_ripples += new_count;
            } else if new_count == 0 {
                xs.remove(droplet_id);
                ys.remove(droplet_id);
                next_mags.remove(droplet_id);
                ripple_freqs.remove(droplet_id);
                ripple_ctrs.remove(droplet_id);
                colors.remove(droplet_id);
                ripple_counts.remove(droplet_id);
            } else {
                ripple_counts[droplet_id] = new_count;
                ripple_ctrs[droplet_id] = ripple_ctr - 1;
                droplet_id += 1;
                new_total_ripples += new_count;
            }
        }
        // assert!(new_total_ripples == ripple_counts.iter().fold(0, |acc, x| acc + x));
        self.droplets.total_ripples = new_total_ripples;
        self.droplets.ripple_mags = new_ripple_mags;
        self.droplets.ripple_max_mags = new_ripple_max_mags;
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
            ripple_mags: _,
            ripple_max_mags: _,
            ripple_counts,
            total_ripples: _,
        } = &mut self.droplets;
        xs.push(x);
        ys.push(y);
        next_mags.push(mag);
        ripple_freqs.push(freq);
        ripple_ctrs.push(0);
        colors.push(color);
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

    pub fn ripple_mags(&self) -> *const DropletStrength {
        self.droplets.ripple_mags.as_ptr()
    }

    pub fn ripple_max_mags(&self) -> *const DropletStrength {
        self.droplets.ripple_max_mags.as_ptr()
    }

    pub fn ripple_counts(&self) -> *const u32 {
        self.droplets.ripple_counts.as_ptr()
    }

    pub fn total_ripples(&self) -> u32 {
        self.droplets.total_ripples
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
}
