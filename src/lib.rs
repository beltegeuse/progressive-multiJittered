extern crate rand;
use rand::prelude::*;

pub trait Generator {
    fn generate(&mut self) -> (f64, f64);
}
pub trait ElementaryElement {
    fn reinitialize(&mut self, expected_size: usize);
    fn update(&mut self, p: (f64, f64));
    fn generate(
        &self,
        i: f64,
        j: f64,
        xhalf: f64,
        yhalf: f64,
        n_float: f64,
        rng: &mut ThreadRng,
    ) -> (f64, f64);
}

// MultiJittered sampler
pub struct HVElementaryElement {
    x_slice: Vec<bool>,
    y_slice: Vec<bool>,
    nn: usize,
}
impl Default for HVElementaryElement {
    fn default() -> Self {
        Self {
            x_slice: vec![],
            y_slice: vec![],
            nn: 0,
        }
    }
}
impl HVElementaryElement {
    fn generate_x(&self, i: f64, xhalf: f64, n_float: f64, rng: &mut ThreadRng) -> f64 {
        loop {
            let v = (i + 0.5 * (xhalf + rng.gen::<f64>())) / n_float;
            assert_ne!(v, 1.0);
            if !self.valid_x(v) {
                return v;
            }
        }
    }
    fn generate_y(&self, j: f64, yhalf: f64, n_float: f64, rng: &mut ThreadRng) -> f64 {
        loop {
            let v = (j + 0.5 * (yhalf + rng.gen::<f64>())) / n_float;
            assert_ne!(v, 1.0);
            if !self.valid_y(v) {
                return v;
            }
        }
    }
    fn valid_x(&self, v: f64) -> bool {
        self.x_slice[(v * self.nn as f64).floor() as usize]
    }
    fn valid_y(&self, v: f64) -> bool {
        self.y_slice[(v * self.nn as f64).floor() as usize]
    }
}
impl ElementaryElement for HVElementaryElement {
    fn reinitialize(&mut self, expected_size: usize) {
        self.nn = expected_size;
        self.x_slice = vec![false; self.nn];
        self.y_slice = vec![false; self.nn];
    }
    fn update(&mut self, (x, y): (f64, f64)) {
        self.x_slice[(x * self.nn as f64).floor() as usize] = true;
        self.y_slice[(y * self.nn as f64).floor() as usize] = true;
    }
    fn generate(
        &self,
        i: f64,
        j: f64,
        xhalf: f64,
        yhalf: f64,
        n_float: f64,
        rng: &mut ThreadRng,
    ) -> (f64, f64) {
        (
            self.generate_x(i, xhalf, n_float, rng),
            self.generate_y(j, yhalf, n_float, rng),
        )
    }
}

// (0,2) Multi-jittered sampler
pub struct ElementaryElement02 {
    ee: Vec<Vec<bool>>,
    nn: usize,
}
impl Default for ElementaryElement02 {
    fn default() -> Self {
        Self { ee: vec![], nn: 0 }
    }
}
impl ElementaryElement02 {
    fn valid(&self, (x, y): (f64, f64)) -> bool {
        let mut x_size = self.nn;
        let mut y_size = 1;
        for e in &self.ee {
            let ix = (x * x_size as f64).floor() as usize;
            let iy = (y * y_size as f64).floor() as usize;
            if e[iy * x_size + ix] {
                return true;
            }
            // Next shape
            x_size = x_size / 2;
            y_size *= 2;
        }
        false
    }
}
impl ElementaryElement for ElementaryElement02 {
    fn reinitialize(&mut self, expected_size: usize) {
        self.nn = expected_size;
        // Count the number of shapes
        // TODO: We can use log2(self.nn*2)
        let mut x_size = expected_size;
        let mut i = 0;
        while x_size != 0 {
            i += 1;
            x_size = x_size / 2;
        }
        self.ee = vec![vec![false; self.nn]; i]
    }
    fn update(&mut self, (x, y): (f64, f64)) {
        let mut x_size = self.nn;
        let mut y_size = 1;
        for e in &mut self.ee {
            let ix = (x * x_size as f64).floor() as usize;
            let iy = (y * y_size as f64).floor() as usize;
            e[iy * x_size + ix] = true;
            // Next shape
            x_size = x_size / 2;
            y_size *= 2;
        }
    }
    fn generate(
        &self,
        i: f64,
        j: f64,
        xhalf: f64,
        yhalf: f64,
        n_float: f64,
        rng: &mut ThreadRng,
    ) -> (f64, f64) {
        loop {
            let vx = (i + 0.5 * (xhalf + rng.gen::<f64>())) / n_float;
            let vy = (j + 0.5 * (yhalf + rng.gen::<f64>())) / n_float;
            if !self.valid((vx, vy)) {
                return (vx, vy);
            }
        }
    }
}

/// === Multi-Jitter
pub struct MultiJittered<T: ElementaryElement + Default> {
    rng: ThreadRng,
    seq: Vec<(f64, f64)>,
    index: usize,
    ee: T,
}
impl<T: ElementaryElement + Default> Default for MultiJittered<T> {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
            seq: vec![],
            index: 0,
            ee: T::default(),
        }
    }
}
impl<T: ElementaryElement + Default> MultiJittered<T> {
    fn extend_sequence_even(&mut self) {
        // Generate the samples on the opposite diagonal only
        // This code is similar to Jittered one
        let n_float = (self.index as f64).sqrt();
        for s in 0..self.index {
            let (old_pt_x, old_pt_y) = self.seq[s];
            let i = (old_pt_x * n_float).floor() as f64;
            let j = (old_pt_y * n_float).floor() as f64;
            let mut xhalf = (2.0 * (old_pt_x * n_float - i)).floor() as i32;
            let mut yhalf = (2.0 * (old_pt_y * n_float - j)).floor() as i32;

            // Diag
            xhalf = 1 - xhalf;
            yhalf = 1 - yhalf;
            self.seq[self.index + s] =
                self.generate_point_slice(i, j, xhalf as f64, yhalf as f64, n_float);
        }
    }

    fn generate_point_slice(
        &mut self,
        i: f64,
        j: f64,
        xhalf: f64,
        yhalf: f64,
        n_float: f64,
    ) -> (f64, f64) {
        let (x, y) = self.ee.generate(i, j, xhalf, yhalf, n_float, &mut self.rng);
        self.ee.update((x, y));
        (x, y)
    }

    fn extend_sequence_odd(&mut self) {
        // Generate on the other quadrant
        let n_float = (self.index as f64).sqrt() as f64;
        let mut halfs = vec![(0, 0); self.index];

        for s in 0..self.index {
            let (old_pt_x, old_pt_y) = self.seq[s];
            let i = (old_pt_x * n_float).floor() as f64;
            let j = (old_pt_y * n_float).floor() as f64;
            let mut xhalf = (2.0 * (old_pt_x * n_float - i)).floor() as i32;
            let mut yhalf = (2.0 * (old_pt_y * n_float - j)).floor() as i32;

            // Choose between the two
            if self.rng.gen::<f64>() > 0.5 {
                xhalf = 1 - xhalf;
            } else {
                yhalf = 1 - yhalf;
            }
            halfs[s] = (xhalf, yhalf);
            self.seq[2 * self.index + s] =
                self.generate_point_slice(i, j, xhalf as f64, yhalf as f64, n_float);
        }

        for s in 0..self.index {
            // Generate the opposite
            let (old_pt_x, old_pt_y) = self.seq[s];
            let i = (old_pt_x * n_float).floor() as f64;
            let j = (old_pt_y * n_float).floor() as f64;
            let xhalf = 1 - halfs[s].0;
            let yhalf = 1 - halfs[s].1;
            self.seq[3 * self.index + s] =
                self.generate_point_slice(i, j, xhalf as f64, yhalf as f64, n_float);
        }
    }
}
impl<T: ElementaryElement + Default> Generator for MultiJittered<T> {
    fn generate(&mut self) -> (f64, f64) {
        if self.index == 0 {
            self.index += 1;
            self.seq.push((self.rng.gen(), self.rng.gen()));
            *self.seq.last().unwrap()
        } else {
            if self.index == self.seq.len() {
                // Resize all
                self.seq.resize(self.index * 4, (0.0, 0.0));

                // here  we will initialize the slice x and y before
                // and them check if there are inside quadrant or not
                // when we generate the points on the fly.
                // as the overlap will be not updated
                self.ee.reinitialize(self.index * 2);
                for i in 0..self.index {
                    self.ee.update(self.seq[i]);
                }
                self.extend_sequence_even();

                // Another subdivision happens
                self.ee.reinitialize(self.index * 4);
                for i in 0..self.index * 2 {
                    self.ee.update(self.seq[i]);
                }
                self.extend_sequence_odd();
            }
            self.index += 1;
            (self.seq[self.index - 1])
        }
    }
}

/// === Jitter
pub struct Jittered {
    rng: ThreadRng,
    seq: Vec<(f64, f64)>,
    index: usize,
}
impl Default for Jittered {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
            seq: vec![],
            index: 0,
        }
    }
}
impl Jittered {
    pub fn reserve(mut self, n: usize) -> Self {
        self.seq.reserve(n);
        self
    }

    fn generate_point(
        &mut self,
        i: f64,
        j: f64,
        xhalf: f64,
        yhalf: f64,
        n_float: f64,
    ) -> (f64, f64) {
        (
            (i + 0.5 * (xhalf + self.rng.gen::<f64>())) / n_float,
            (j + 0.5 * (yhalf + self.rng.gen::<f64>())) / n_float,
        )
    }

    fn extend_sequence(&mut self) {
        let n_float = (self.index as f64).sqrt();
        self.seq.resize(self.index * 4, (0.0, 0.0));
        for s in 0..self.index {
            let (old_pt_x, old_pt_y) = self.seq[s];
            let i = (old_pt_x * n_float).floor() as f64;
            let j = (old_pt_y * n_float).floor() as f64;
            let mut xhalf = (2.0 * (old_pt_x * n_float - i)).floor() as i32;
            let mut yhalf = (2.0 * (old_pt_y * n_float - j)).floor() as i32;

            // Diag
            xhalf = 1 - xhalf;
            yhalf = 1 - yhalf;
            self.seq[self.index + s] =
                self.generate_point(i, j, xhalf as f64, yhalf as f64, n_float);

            // Choose between the two
            if self.rng.gen::<f64>() > 0.5 {
                xhalf = 1 - xhalf;
            } else {
                yhalf = 1 - yhalf;
            }
            self.seq[2 * self.index + s] =
                self.generate_point(i, j, xhalf as f64, yhalf as f64, n_float);

            // Opposite
            xhalf = 1 - xhalf;
            yhalf = 1 - yhalf;
            self.seq[3 * self.index + s] =
                self.generate_point(i, j, xhalf as f64, yhalf as f64, n_float);
        }
    }
}
impl Generator for Jittered {
    fn generate(&mut self) -> (f64, f64) {
        if self.index == 0 {
            self.index += 1;
            self.seq.push((self.rng.gen(), self.rng.gen()));
            *self.seq.last().unwrap()
        } else {
            if self.index == self.seq.len() {
                self.extend_sequence();
            }
            self.index += 1;
            (self.seq[self.index - 1])
        }
    }
}

/// === Uniform
pub struct Uniform {
    rng: ThreadRng,
}
impl Default for Uniform {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}
impl Generator for Uniform {
    fn generate(&mut self) -> (f64, f64) {
        (self.rng.gen(), self.rng.gen())
    }
}
