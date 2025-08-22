use crate::drawing::axis;
use crate::ir::axis::ticks::{Formatter, Locator, Ticks};

pub fn locate(locator: &Locator, ab: axis::NumBounds) -> Vec<f64> {
    match locator {
        Locator::Auto => MaxN::new_auto().ticks(ab),
        Locator::MaxN { bins, steps } => {
            let ticker = MaxN::new(*bins, steps.as_slice());
            ticker.ticks(ab)
        }
        Locator::PiMultiple { bins } => {
            let ticker = MaxN::new_pi(*bins);
            ticker.ticks(ab)
        }
    }
}

pub fn locate_minor(locator: &Locator, ab: axis::NumBounds) -> Vec<f64> {
    match locator {
        Locator::Auto => MaxN::new_auto_minor().ticks(ab),
        _ => todo!("minor locators"),
    }
}

const AUTO_BINS: u32 = 10;
const AUTO_BINS_MINOR: u32 = 50;
const AUTO_STEPS: &[f64] = &[1.0, 2.0, 2.5, 5.0];

const PI: f64 = std::f64::consts::PI;
const PI_STEPS: &[f64] = &[PI / 8.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI];

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

    fn new_auto_minor() -> Self {
        Self::new(AUTO_BINS_MINOR, AUTO_STEPS)
    }

    fn new_pi(bins: u32) -> Self {
        Self::new(bins, PI_STEPS)
    }

    fn ticks(&self, ab: axis::NumBounds) -> Vec<f64> {
        let target_step = ab.span() / self.bins as f64;

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

        let vmin = (ab.start() / step).floor() * step;

        let edge = MaxNEdgeInteger { step };
        let low = edge.largest_le(ab.start() - vmin);
        let high = edge.smallest_ge(ab.end() - vmin);

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

struct MaxNEdgeInteger {
    step: f64,
}

fn is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}

impl MaxNEdgeInteger {
    fn largest_le(&self, value: f64) -> f64 {
        let (d, m) = (value.div_euclid(self.step), value % self.step);
        if is_close(m / self.step, 1.0) {
            d + 1.0
        } else {
            d
        }
    }
    fn smallest_ge(&self, value: f64) -> f64 {
        let (d, m) = (value.div_euclid(self.step), value % self.step);
        if is_close(m / self.step, 0.0) {
            d
        } else {
            d + 1.0
        }
    }
}

pub fn label_formatter(ticks: &Ticks, ab: axis::NumBounds) -> Box<dyn LabelFormatter> {
    match ticks.formatter() {
        Formatter::Auto => auto_label_formatter(ticks.locator(), ab),
        Formatter::Prec(prec) => Box::new(PrecLabelFormat(*prec)),
        Formatter::Percent => Box::new(PercentLabelFormat),
    }
}

fn auto_label_formatter(locator: &Locator, _ab: axis::NumBounds) -> Box<dyn LabelFormatter> {
    match locator {
        Locator::PiMultiple { .. } => Box::new(PiMultipleLabelFormat { prec: 2 }),
        Locator::Auto => Box::new(PrecLabelFormat(2)),
        _ => todo!(),
    }
}

pub trait LabelFormatter {
    fn axis_annotation(&self) -> Option<&str> {
        None
    }
    fn format_label(&self, data: f64) -> String;
}

struct PrecLabelFormat(usize);

impl LabelFormatter for PrecLabelFormat {
    fn format_label(&self, data: f64) -> String {
        format!("{data:.*}", self.0)
    }
}

struct PiMultipleLabelFormat {
    prec: usize,
}

impl LabelFormatter for PiMultipleLabelFormat {
    fn axis_annotation(&self) -> Option<&str> {
        Some("\u{00d7} π")
    }
    fn format_label(&self, data: f64) -> String {
        let val = data / PI;
        format!("{val:.*}", self.prec)
    }
}

struct PercentLabelFormat;

impl LabelFormatter for PercentLabelFormat {
    fn format_label(&self, data: f64) -> String {
        format!("{:.0}%", data * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawing::axis;

    fn is_close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn slice_contains_sample_f64(slice: &[f64], sample: &[f64], tol: f64) -> bool {
        let fst_sample = sample[0];
        let idx = slice.iter().position(|x| is_close(*x, fst_sample, tol));
        if idx.is_none() {
            return false;
        }
        let idx = idx.unwrap();
        if slice.len() - idx < sample.len() {
            return false;
        }
        slice
            .iter()
            .skip(idx)
            .zip(sample.iter())
            .all(|(a, b)| is_close(*a, *b, tol))
    }

    macro_rules! assert_contains_f64 {
        ($slice:expr, $sample:expr, $tol:expr) => {
            assert!(
                slice_contains_sample_f64(&$slice, &$sample, $tol),
                "Assertion failed: Slice doesn't contain sample.\nSlice:  {:?}\nSample: {:?}",
                $slice,
                $sample
            );
        };
        ($slice:expr, $sample:expr) => {
            assert_contains_f64!($slice, $sample, 1e-8);
        };
    }

    #[test]
    fn test_ticks_loc_auto() {
        let locator = MaxN::new_auto();

        let ticks = locator.ticks(axis::NumBounds::from((-1.0, 1.0)));
        let expected = vec![-1.0, -0.8, -0.6, -0.4, -0.2, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        assert_contains_f64!(ticks, expected);

        let ticks = locator.ticks(axis::NumBounds::from((0.0, 0.195)));
        let expected = vec![
            0.0, 0.02, 0.04, 0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.18, 0.2,
        ];
        assert_contains_f64!(ticks, expected);

        let ticks = locator.ticks(axis::NumBounds::from((0.005, 0.195)));
        let expected = vec![
            0.0, 0.02, 0.04, 0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.18, 0.2,
        ];
        assert_contains_f64!(ticks, expected);
    }

    #[test]
    fn test_ticks_loc_pi_multiple() {
        use std::f64::consts::PI;

        let locator = MaxN::new_pi(8);
        let ticks = locator.ticks(axis::NumBounds::from((0.0, 2.0 * PI)));
        let expected = vec![
            0.0,
            0.25 * PI,
            0.5 * PI,
            0.75 * PI,
            1.0 * PI,
            1.25 * PI,
            1.5 * PI,
            1.75 * PI,
            2.0 * PI,
        ];
        assert_contains_f64!(ticks, expected);

        let locator = MaxN::new_pi(4);
        let ticks = locator.ticks(axis::NumBounds::from((0.0, 2.0 * PI)));
        let expected = vec![0.0, 0.5 * PI, 1.0 * PI, 1.5 * PI, 2.0 * PI];
        assert_contains_f64!(ticks, expected);
    }
}
