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
struct Droplet {
    x: u32,
    y: u32,
    /// The strength of the droplet, i.e. the radius of the first and larget ripple
    mag: u32,
    /// The RGB hex code of this droplet (only the lower 24 bits are used)
    color: u32,
    /// The number of iterations that pass before a new ripple is generated
    ripple_freq: u32,
}

impl Droplet {
    /// Generates a `Droplet` of 1 less energy if `global_ctr % ripple_freq` is `0`.
    /// If this `Droplet` already has 0 energy, then it drops itself and returns `None`.
    pub fn lose_energy(&self, global_ctr: u64) -> Option<Droplet> {
        if self.mag <= 1 {
            None
        } else if global_ctr % (self.ripple_freq as u64) == 0 {
            Some(Droplet {
                mag: self.mag - 1,
                ..*self
            })
        } else {
            Some(Droplet { ..*self })
        }
    }

    /// The number of cycles left in the lifetime of this `Droplet`, assuming the global
    /// pond counter is currently at 0.
    //
    // A droplet of magnitude 2 and ripple_freq 3 lasts for 1 cycle if the counter starts at 0
    // ..............
    // 1x
    // A droplet of magnitude 3 and ripple_freq 2 lasts for 5 cycles if the counter starts at 0
    // 3221x
    /// The general formula is mag * (ripple_freq - 1) + 1
    /// A droplet spends ripple_freq cycles at any given energy level; the droplet is
    /// dropped after one cycle at energy level of 1 so we subtract 1 from mag and
    /// add one back.
    #[cfg(test)]
    pub fn exp_lifetime(&self) -> u64 {
        ((self.mag - 1) * self.ripple_freq + 1) as u64
    }
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

impl Ripples {
    pub fn new() -> Ripples {
        Ripples {
            xs: Vec::new(),
            ys: Vec::new(),
            mags: Vec::new(),
            max_mags: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn add_ripple(&mut self, droplet: &Droplet) {
        let &Droplet {
            x,
            y,
            mag,
            color,
            ripple_freq: _,
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
    counter: u64,
}

#[wasm_bindgen]
impl Pond {
    pub fn new(width: u32, height: u32) -> Pond {
        Pond {
            width,
            height,
            droplets: Vec::new(),
            ripples: Ripples::new(),
            counter: 0u64,
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
        // Need to increase magnitude by 1 if ripple is not dead
        let mut mags_copy: Vec<u32> = mags.iter().map(|m| m + 1).collect();
        let mut i = 0;
        while i != mags_copy.len() {
            if mags_copy[i] > max_mags[i] {
                // Remove inert ripples
                xs.remove(i);
                ys.remove(i);
                // Note that mags_copy is referenced here, since this is what's new
                mags_copy.remove(i);
                max_mags.remove(i);
                colors.remove(i);
            } else {
                i += 1;
            }
        }
        self.ripples.mags = mags_copy;
        // Add fresh ripples, and remove dead droplets at the same time
        let mut new_droplets = Vec::with_capacity(self.droplets.len());
        let ctr = self.counter;
        for droplet in &self.droplets {
            if ctr % (droplet.ripple_freq as u64) == 0 {
                self.ripples.add_ripple(droplet);
            }
            // TODO refactor out of this method
            if let Some(new_droplet) = droplet.lose_energy(ctr) {
                new_droplets.push(new_droplet);
            }
        }
        self.droplets = new_droplets;
        self.counter += 1;
    }

    #[cfg(test)]
    fn add_test_droplet(&mut self, droplet: &Droplet) {
        self.add_droplet(
            droplet.x,
            droplet.y,
            droplet.mag,
            droplet.color,
            droplet.ripple_freq,
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn single_droplet_dequeue_test(mag: u32, ripple_freq: u32) {
        let mut pond = Pond::new(100, 100);
        let droplet = Droplet {
            x: 0,
            y: 0,
            mag,
            color: 0xFFFF,
            ripple_freq,
        };
        pond.add_test_droplet(&droplet);
        let exp_lifetime = if mag == 1 {
            1
        } else {
            // If we start on cycle 0, the magnitude is decremented after 1 cycle
            droplet.exp_lifetime() - (ripple_freq as u64) + 1
        };
        while pond.droplets.len() > 0 {
            pond.tick();
        }
        assert_eq!(
            exp_lifetime, pond.counter,
            "testing with mag={}, ripple_freq={}",
            mag, ripple_freq
        );
    }

    /// Make sure a droplet lasts for the appropriate number of cycles
    #[test]
    fn single_droplet_dequeue() {
        single_droplet_dequeue_test(1, 2);
        single_droplet_dequeue_test(2, 2);
        single_droplet_dequeue_test(3, 7);
        single_droplet_dequeue_test(5, 10);
    }
}
