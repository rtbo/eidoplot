use crate::data;
use crate::drawing::{Categories, Error, axis};
use crate::ir::axis::ticks::{Formatter, Locator, Ticks, TimeFormatter, TimeLocator};
use crate::ir::axis::{LogScale, Scale};
use crate::time::{DateTime, DateTimeComps, TimeDelta};

pub fn locate_num(
    locator: &Locator,
    nb: axis::NumBounds,
    scale: &Scale,
) -> Result<Vec<f64>, Error> {
    match (locator, scale) {
        (Locator::Auto, Scale::Auto | Scale::Linear { .. }) => Ok(MaxN::new_auto().ticks(nb)),
        (Locator::Auto, Scale::Log(LogScale { base, .. })) => {
            Ok(LogLocator::new_major(*base).ticks(nb))
        }
        (Locator::MaxN { bins, steps }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new(*bins, steps.as_slice());
            Ok(ticker.ticks(nb))
        }
        (Locator::PiMultiple { bins }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new_pi(*bins);
            Ok(ticker.ticks(nb))
        }
        (Locator::Log { base, .. }, Scale::Auto) => Ok(LogLocator::new_major(*base).ticks(nb)),
        (Locator::Log { base: loc_base, .. }, Scale::Log(LogScale { base, .. }))
            if loc_base == base =>
        {
            Ok(LogLocator::new_major(*base).ticks(nb))
        }
        _ => Err(Error::InconsistentIr(format!(
            "Unsupported locator/scale combination: {:?}/{:?}",
            locator, scale
        ))),
    }
}

pub fn locate_minor(
    locator: &Locator,
    nb: axis::NumBounds,
    scale: &Scale,
) -> Result<Vec<f64>, Error> {
    match (locator, scale) {
        (Locator::Auto, Scale::Auto | Scale::Linear { .. }) => Ok(MaxN::new_auto_minor().ticks(nb)),
        (Locator::Auto, Scale::Log(LogScale { base, .. })) => {
            Ok(LogLocator::new_minor(*base).ticks(nb))
        }
        (Locator::MaxN { bins, steps }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new(*bins, steps.as_slice());
            Ok(ticker.ticks(nb))
        }
        (Locator::PiMultiple { bins }, Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new_pi(*bins);
            Ok(ticker.ticks(nb))
        }
        (Locator::Log { base, .. }, Scale::Auto) => Ok(LogLocator::new_minor(*base).ticks(nb)),
        (Locator::Log { base: loc_base, .. }, Scale::Log(LogScale { base, .. }))
            if loc_base == base =>
        {
            Ok(LogLocator::new_minor(*base).ticks(nb))
        }
        _ => Err(Error::InconsistentIr(format!(
            "Unsupported locator/scale combination: {:?}/{:?}",
            locator, scale
        ))),
    }
}

pub fn locate_time(locator: &Locator, tb: axis::TimeBounds) -> Result<Vec<DateTime>, Error> {
    match locator {
        Locator::Auto | Locator::Time(TimeLocator::Auto) => {
            let span = tb.span();

            // heuristics to pick a locator that will yield ~ 5 to 10 ticks
            let locator = if span > TimeDelta::from_days(5.0 * 365.0) {
                let years = span.seconds() / (10.0 * 365.0 * 86400.0);
                Locator::Time(TimeLocator::Years((years as u32).max(1)))
            } else if span > TimeDelta::from_days(5.0 * 30.0) {
                let months = span.seconds() / (10.0 * 30.0 * 86400.0);
                Locator::Time(TimeLocator::Months((months as u32).max(1)))
            } else if span > TimeDelta::from_days(5.0 * 7.0) {
                let weeks = span.seconds() / (10.0 * 7.0 * 86400.0);
                Locator::Time(TimeLocator::Weeks((weeks as u32).max(1)))
            } else if span > TimeDelta::from_days(5.0) {
                let days = span.seconds() / (10.0 * 86400.0);
                Locator::Time(TimeLocator::Days((days as u32).max(1)))
            } else if span > TimeDelta::from_hours(5.0) {
                let hours = span.seconds() / (10.0 * 3600.0);
                Locator::Time(TimeLocator::Hours((hours as u32).max(1)))
            } else if span > TimeDelta::from_minutes(5.0) {
                let minutes = span.seconds() / (10.0 * 60.0);
                Locator::Time(TimeLocator::Minutes((minutes as u32).max(1)))
            } else if span > TimeDelta::from_seconds(5.0) {
                let seconds = span.seconds() / 10.0;
                Locator::Time(TimeLocator::Seconds((seconds as u32).max(1)))
            } else {
                let micro = span.seconds() * 1_000_000.0 / 10.0;
                Locator::Time(TimeLocator::Micros((micro as u32).max(1)))
            };
            locate_time(&locator, tb)
        }
        &Locator::Time(TimeLocator::Years(n)) => {
            let start = tb.start().to_comps();
            let end = tb.end().to_comps();
            let mut dt = DateTimeComps {
                year: start.year,
                ..DateTimeComps::epoch()
            };
            let mut res = Vec::new();
            while dt < end {
                res.push(dt.try_into().unwrap());
                dt.year += n as i32;
            }
            Ok(res)
        }
        &Locator::Time(TimeLocator::Months(n)) => {
            let start = tb.start().to_comps();
            let end = tb.end().to_comps();
            let mut dt = DateTimeComps {
                year: start.year,
                month: start.month,
                ..DateTimeComps::epoch()
            };
            let mut res = Vec::new();
            while dt < end {
                res.push(dt.try_into().unwrap());
                dt.month += n;
                if dt.month > 12 {
                    dt.year += 1;
                    dt.month -= 12;
                }
            }
            Ok(res)
        }
        &Locator::Time(TimeLocator::Weeks(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start();
            let end = tb.end();
            let td = TimeDelta::from_seconds(7.0 * 24.0 * 3600.0) * n as f64;
            Ok(locate_time_even(start, end, td))
        }
        &Locator::Time(TimeLocator::Days(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                ..DateTimeComps::epoch()
            };
            let td = TimeDelta::from_seconds(24.0 * 3600.0) * n as f64;
            let end = tb.end();
            Ok(locate_time_even(start.try_into().unwrap(), end, td))
        }
        &Locator::Time(TimeLocator::Hours(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                ..DateTimeComps::epoch()
            };
            let end = tb.end();
            let td = TimeDelta::from_seconds(3600.0) * n as f64;
            Ok(locate_time_even(start.try_into().unwrap(), end, td))
        }
        &Locator::Time(TimeLocator::Minutes(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                ..DateTimeComps::epoch()
            };
            let end = tb.end();
            let td = TimeDelta::from_seconds(60.0) * n as f64;
            Ok(locate_time_even(start.try_into().unwrap(), end, td))
        }
        &Locator::Time(TimeLocator::Seconds(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                ..DateTimeComps::epoch()
            };
            let end = tb.end();
            let td = TimeDelta::from_seconds(1.0) * n as f64;
            Ok(locate_time_even(start.try_into().unwrap(), end, td))
        }
        &Locator::Time(TimeLocator::Micros(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                ..DateTimeComps::epoch()
            };
            let end = tb.end();
            let td = TimeDelta::from_seconds(1E-6) * n as f64;
            Ok(locate_time_even(start.try_into().unwrap(), end, td))
        }
        _ => Err(Error::InconsistentIr(format!(
            "Inconsistent ticks locator for time axis: {locator:?}"
        ))),
    }
}

fn locate_time_even(start: DateTime, end: DateTime, td: TimeDelta) -> Vec<DateTime> {
    let mut res = Vec::new();
    let mut cur = start;
    while cur < end {
        res.push(cur);
        cur += td;
    }
    res
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
        Some(Formatter::Auto) if scale.is_shared() => Box::new(NullFormat),
        Some(Formatter::Auto | Formatter::SharedAuto) => {
            auto_label_formatter(ticks.locator(), ab, scale)
        }
        Some(Formatter::Prec(prec)) => Box::new(PrecLabelFormat(*prec)),
        Some(Formatter::Percent) => Box::new(PercentLabelFormat),
        None => Box::new(NullFormat),
        _ => todo!(),
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

pub fn time_label_formatter(
    ticks: &Ticks,
    tb: axis::TimeBounds,
    scale: &Scale,
) -> Result<Box<dyn LabelFormatter>, Error> {
    match ticks.formatter() {
        Some(Formatter::Auto) if scale.is_shared() => Ok(Box::new(NullFormat)),
        Some(Formatter::Auto | Formatter::SharedAuto) => {
            auto_time_label_formatter(tb)
        }
        Some(Formatter::Time(TimeFormatter::Auto)) => {
            auto_time_label_formatter(tb)
        },
        Some(Formatter::Time(TimeFormatter::DateTime)) => {
            Ok(Box::new(TimeLabelFormat { fmt: "%Y-%m-%d %H:%M:%S".to_string() }))
        },
        Some(Formatter::Time(TimeFormatter::Date)) => {
            Ok(Box::new(TimeLabelFormat { fmt: "%Y-%m-%d".to_string() }))
        }
        Some(Formatter::Time(TimeFormatter::Time)) => {
            Ok(Box::new(TimeLabelFormat { fmt: "%H:%M:%S".to_string() }))
        }
        Some(Formatter::Time(TimeFormatter::Custom(fmt))) => {
            Ok(Box::new(TimeLabelFormat { fmt: fmt.clone() }))
        }
        None => Ok(Box::new(NullFormat)),
        _ => todo!(),
    }
}

fn auto_time_label_formatter(
    tb: axis::TimeBounds,
) -> Result<Box<dyn LabelFormatter>, Error> {
    let start_date = tb.start().to_date();
    let end_date = tb.end().to_date();
    let span = tb.span();

    let fmt = if start_date == end_date {
        if span > TimeDelta::from_minutes(10.0) {
            "%H:%M"
        } else if span > TimeDelta::from_seconds(10.0) {
            "%H:%M:%S"
        } else {
            "%H:%M:%S%.f"
        }
    } else if span < TimeDelta::from_days(10.0) {
        "%Y-%m-%d %H:%M"
    } else if span < TimeDelta::from_days(10.0 * 30.0) {
        "%Y-%m-%d"
    } else {
        "%Y-%m"
    };

    Ok(Box::new(TimeLabelFormat { fmt: fmt.to_string() }))
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

struct TimeLabelFormat {
    fmt: String,
}

impl LabelFormatter for TimeLabelFormat {
    fn format_label(&self, data: data::Sample) -> String {
        let dt = data.as_time().unwrap();
        format!("{}", dt.fmt_to_string(&self.fmt))
    }
}

struct NullFormat;

impl LabelFormatter for NullFormat {
    fn format_label(&self, _: data::Sample) -> String {
        String::new()
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
