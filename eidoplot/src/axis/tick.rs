use crate::data;

#[derive(Debug, Clone)]
pub enum Locator {
    Auto,
    MaxN { bins: u32, steps: Vec<f64> },
    PiMultiple { bins: u32 },
}

impl Default for Locator {
    fn default() -> Self {
        Locator::Auto
    }
}

impl Locator {
    pub fn ticks(&self, db: data::Bounds) -> Vec<f64> {
        match self {
            Locator::Auto => MaxN::new_auto().ticks(db),
            Locator::MaxN { bins, steps } => {
                let ticker = MaxN::new(*bins, steps.as_slice());
                ticker.ticks(db)
            }
            Locator::PiMultiple { bins } => {
                let ticker = MaxN::new_pi(*bins);
                ticker.ticks(db)
            }
        }
    }
}

const AUTO_BINS: u32 = 10;
const AUTO_STEPS: &[f64] = &[1.0, 2.0, 2.5, 5.0];

const PI: f64 = std::f64::consts::PI;
const PI_STEPS: &[f64] = &[PI / 8.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0];

struct MaxN<'a> {
    bins: u32,
    steps: &'a [f64],
}

impl<'a> MaxN<'a> {
    fn new(bins: u32, steps: &'a [f64]) -> Self {
        Self { bins, steps }
    }

    fn new_auto() -> Self {
        Self::new(AUTO_BINS, AUTO_STEPS)
    }

    fn new_pi(bins: u32) -> Self {
        Self::new(bins, PI_STEPS)
    }

    fn ticks(&self, db: data::Bounds) -> Vec<f64> {
        let target_step = db.span() / self.bins as f64;

        // getting quite about where we need to be
        let scale = 10f64.powf(target_step.log10().div_euclid(1.0));
        assert!(scale > 0.0);

        let step = {
            let mut stepper = MaxNStepper::new(self.steps, scale);
            while stepper.step() > target_step {
                stepper.next_smaller();
            }
            while stepper.step() < target_step {
                stepper.next_bigger();
            }
            stepper.step()
        };

        let vmin = (db.min() / step).floor() * step;

        let edge = MaxNEdge { step };
        let low = edge.largest_le(db.min() - vmin);
        let high = edge.smallest_ge(db.max() - vmin);

        let mut ticks = Vec::with_capacity((high - low + 1.0) as usize);
        let mut val = low;
        while val <= high {
            ticks.push(vmin + val * step);
            val += 1.0;
        }
        ticks
    }
}

#[derive(Debug, Clone, Copy)]
struct MaxNStepper<'a> {
    steps: &'a [f64],
    idx: usize,
    scale: f64,
}

impl<'a> MaxNStepper<'a> {
    fn new(steps: &'a [f64], scale: f64) -> Self {
        MaxNStepper {
            steps,
            scale,
            idx: 0,
        }
    }

    fn step(&self) -> f64 {
        self.steps[self.idx] * self.scale
    }

    fn next_smaller(&mut self) {
        if self.idx == 0 {
            self.idx = self.steps.len();
            self.scale *= 0.1;
        }
        self.idx -= 1;
    }

    fn next_bigger(&mut self) {
        self.idx += 1;
        if self.idx == self.steps.len() {
            self.idx = 0;
            self.scale *= 10.0;
        }
    }
}

struct MaxNEdge {
    step: f64,
}

impl MaxNEdge {
    fn largest_le(&self, value: f64) -> f64 {
        let (d, m) = (value / self.step, value % self.step);
        if float_eq::float_eq!(m / self.step, 1.0, abs <= 1e-10) {
            d + 1.0
        } else {
            d
        }
    }
    fn smallest_ge(&self, value: f64) -> f64 {
        let (d, m) = (value / self.step, value % self.step);
        if float_eq::float_eq!(m / self.step, 0.0, abs <= 1e-10) {
            d
        } else {
            d + 1.0
        }
    }
}
