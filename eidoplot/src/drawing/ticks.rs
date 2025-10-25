use crate::data;
use crate::drawing::{Categories, axis};
use crate::ir::axis::ticks::{Formatter, Locator, Ticks};
use crate::ir::axis::{LogScale, Scale};

pub fn locate_num(locator: &Locator, nb: axis::NumBounds, scale: &Scale) -> Vec<f64> {
    match (locator, scale) {
        (Locator::Auto, Scale::Auto | Scale::Linear { .. }) => MaxN::new_auto().ticks(nb),
        (Locator::Auto, Scale::Log(LogScale { base, .. })) => {
            LogLocator::new_major(*base).ticks(nb)
        }
        (Locator::MaxN { bins, steps }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new(*bins, steps.as_slice());
            ticker.ticks(nb)
        }
        (Locator::PiMultiple { bins }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new_pi(*bins);
            ticker.ticks(nb)
        }
        (Locator::Log { base, .. }, Scale::Auto) => LogLocator::new_major(*base).ticks(nb),
        (Locator::Log { base: loc_base, .. }, Scale::Log(LogScale { base, .. }))
            if loc_base == base =>
        {
            LogLocator::new_major(*base).ticks(nb)
        }
        _ => panic!(
            "Unsupported locator/scale combination: {:?}/{:?}\n(FIXME: error check during IR construction)",
            locator, scale
        ),
    }
}

pub fn locate_minor(locator: &Locator, nb: axis::NumBounds, scale: &Scale) -> Vec<f64> {
    match (locator, scale) {
        (Locator::Auto, Scale::Auto | Scale::Linear { .. }) => MaxN::new_auto_minor().ticks(nb),
        (Locator::Auto, Scale::Log(LogScale { base, .. })) => {
            LogLocator::new_minor(*base).ticks(nb)
        }
        (Locator::MaxN { bins, steps }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new(*bins, steps.as_slice());
            ticker.ticks(nb)
        }
        (Locator::PiMultiple { bins }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new_pi(*bins);
            ticker.ticks(nb)
        }
        (Locator::Log { base, .. }, Scale::Auto) => LogLocator::new_minor(*base).ticks(nb),
        (Locator::Log { base: loc_base, .. }, Scale::Log(LogScale { base, .. }))
            if loc_base == base =>
        {
            LogLocator::new_minor(*base).ticks(nb)
        }
        _ => panic!(
            "Unsupported locator/scale combination: {:?}/{:?}\n(FIXME: error check during IR construction)",
            locator, scale
        ),
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

    fn ticks(&self, nb: axis::NumBounds) -> Vec<f64> {
        let target_step = nb.span() / self.bins as f64;

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

        let vmin = (nb.start() / step).floor() * step;

        let edge = MaxNEdgeInteger { step };
        let low = edge.largest_le(nb.start() - vmin);
        let high = edge.smallest_ge(nb.end() - vmin);

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

struct LogLocator {
    base: f64,
    include_minor: bool,
}

impl LogLocator {
    fn new_major(base: f64) -> Self {
        Self {
            base,
            include_minor: false,
        }
    }

    fn new_minor(base: f64) -> Self {
        Self {
            base,
            include_minor: true,
        }
    }

    fn ticks(&self, nb: axis::NumBounds) -> Vec<f64> {
        assert!((nb.start() > 0.0 && nb.end() > 0.0) || (nb.start() < 0.0 && nb.end() < 0.0));

        let (min, max) = if nb.start() < nb.end() {
            (nb.start(), nb.end())
        } else {
            (nb.end(), nb.start())
        };

        // Compute the integer exponents that cover the range
        let min_exp = min.log(self.base).ceil() as i32;
        let max_exp = max.log(self.base).floor() as i32;

        let mut ticks = Vec::new();
        for exp in min_exp..=max_exp {
            let tick = self.base.powi(exp);
            if self.include_minor {
                let minor_incr = tick / self.base;
                let mut minor_tick = minor_incr;
                while minor_tick < tick {
                    if is_close(minor_tick, tick) {
                        break;
                    }
                    ticks.push(minor_tick);
                    minor_tick += minor_incr;
                }
            }
            ticks.push(tick);
        }
        ticks
    }
}

pub fn num_label_formatter(
    ticks: &Ticks,
    ab: axis::NumBounds,
    scale: &Scale,
) -> Box<dyn LabelFormatter> {
    match ticks.formatter() {
        Formatter::Auto => auto_label_formatter(ticks.locator(), ab, scale),
        Formatter::Prec(prec) => Box::new(PrecLabelFormat(*prec)),
        Formatter::Percent => Box::new(PercentLabelFormat),
    }
}

fn auto_label_formatter(
    locator: &Locator,
    _ab: axis::NumBounds,
    scale: &Scale,
) -> Box<dyn LabelFormatter> {
    match (locator, scale) {
        (Locator::PiMultiple { .. }, _) => Box::new(PiMultipleLabelFormat { prec: 2 }),
        (Locator::Auto, Scale::Log(LogScale { base, .. })) if *base == 10.0 => {
            Box::new(SciLabelFormat)
        }
        (Locator::Auto, _) => Box::new(PrecLabelFormat(2)),
        _ => todo!(),
    }
}

pub trait LabelFormatter {
    fn axis_annotation(&self) -> Option<&str> {
        None
    }
    fn format_label(&self, data: data::Sample) -> String;
}

impl LabelFormatter for Categories {
    fn format_label(&self, data: data::Sample) -> String {
        let cat = data.as_cat().expect("Should be a category");
        self.iter()
            .find(|c| *c == cat)
            .map(str::to_string)
            .unwrap_or_default()
    }
}

struct PrecLabelFormat(usize);

impl LabelFormatter for PrecLabelFormat {
    fn format_label(&self, data: data::Sample) -> String {
        let data = data.as_num().unwrap();
        format!("{data:.*}", self.0)
    }
}

struct SciLabelFormat;

impl LabelFormatter for SciLabelFormat {
    fn format_label(&self, data: data::Sample) -> String {
        let data = data.as_num().unwrap();
        format!("{data:e}")
    }
}

struct PiMultipleLabelFormat {
    prec: usize,
}

impl LabelFormatter for PiMultipleLabelFormat {
    fn axis_annotation(&self) -> Option<&str> {
        Some("\u{00d7} π")
    }
    fn format_label(&self, data: data::Sample) -> String {
        let data = data.as_num().unwrap();
        let val = data / PI;
        format!("{val:.*}", self.prec)
    }
}

struct PercentLabelFormat;

impl LabelFormatter for PercentLabelFormat {
    fn format_label(&self, data: data::Sample) -> String {
        let data = data.as_num().unwrap();
        format!("{:.0}%", data * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawing::axis;
    use crate::tests::Near;

    fn contains_near<N>(slice: &[f64], sample: &[f64], near: N) -> bool
    where
        N: Fn(f64, f64) -> bool,
    {
        let fst_sample = sample[0];
        let idx = slice.iter().position(|x| near(*x, fst_sample));
        if idx.is_none() {
            return false;
        }
        let idx = idx.unwrap();
        if slice.len() - idx < sample.len() {
            return false;
        }
        let slice = &slice[idx..idx + sample.len()];
        slice.iter().zip(sample.iter()).all(|(a, b)| near(*a, *b))
    }

    macro_rules! assert_contains_near {
        (abs, $slice:expr, $sample:expr, $tol:expr) => {
            assert!(
                contains_near(&$slice, &$sample, |a, b| a.near_abs(&b, $tol)),
                "Assertion failed: Slice doesn't contain sample.\nSlice:  {:?}\nSample: {:?}",
                $slice,
                $sample
            );
        };
        (abs, $slice:expr, $sample:expr) => {
            assert_contains_near!(abs, $slice, $sample, 1e-8);
        };
        (rel, $slice:expr, $sample:expr, $err:expr) => {
            assert!(
                contains_near(&$slice, &$sample, |a, b| a.near_rel(&b, $err)),
                "Assertion failed: Slice doesn't contain sample.\nSlice:  {:?}\nSample: {:?}",
                $slice,
                $sample
            );
        };
        (rel, $slice:expr, $sample:expr) => {
            assert_contains_near!(rel, $slice, $sample, 1e-8);
        };
    }

    #[test]
    fn test_ticks_loc_auto() {
        let locator = MaxN::new_auto();

        let ticks = locator.ticks(axis::NumBounds::from((-1.0, 1.0)));
        let expected = vec![-1.0, -0.8, -0.6, -0.4, -0.2, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
        assert_contains_near!(abs, ticks, expected);

        let ticks = locator.ticks(axis::NumBounds::from((0.0, 0.195)));
        let expected = vec![
            0.0, 0.02, 0.04, 0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.18, 0.2,
        ];
        assert_contains_near!(abs, ticks, expected);

        let ticks = locator.ticks(axis::NumBounds::from((0.005, 0.195)));
        let expected = vec![
            0.0, 0.02, 0.04, 0.06, 0.08, 0.1, 0.12, 0.14, 0.16, 0.18, 0.2,
        ];
        assert_contains_near!(abs, ticks, expected);
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
        assert_contains_near!(abs, ticks, expected);

        let locator = MaxN::new_pi(4);
        let ticks = locator.ticks(axis::NumBounds::from((0.0, 2.0 * PI)));
        let expected = vec![0.0, 0.5 * PI, 1.0 * PI, 1.5 * PI, 2.0 * PI];
        assert_contains_near!(abs, ticks, expected);
    }
}
