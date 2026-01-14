use std::fmt::Debug;
use std::sync::Arc;

use crate::data;
#[cfg(feature = "time")]
use crate::des::axis::ticks::{
    DateTimeFormatter, DateTimeLocator, TimeDeltaFormatter, TimeDeltaLocator,
};
use crate::des::axis::ticks::{Formatter, Locator, Ticks};
use crate::des::axis::{LogScale, Scale};
use crate::drawing::{Categories, Error, axis};
#[cfg(feature = "time")]
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
        (Locator::MaxN(locator), Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new(locator.bins, locator.steps.as_slice());
            Ok(ticker.ticks(nb))
        }
        (Locator::PiMultiple(locator), Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new_pi(locator.bins);
            Ok(ticker.ticks(nb))
        }
        (Locator::Log(locator), Scale::Auto) => Ok(LogLocator::new_major(locator.base).ticks(nb)),
        (Locator::Log(locator), Scale::Log(LogScale { base, .. })) if locator.base == *base => {
            Ok(LogLocator::new_major(*base).ticks(nb))
        }
        #[cfg(feature = "time")]
        (Locator::TimeDelta(loc), Scale::Auto | Scale::Linear { .. }) => {
            locate_timedelta_num(loc, nb)
        }
        _ => Err(Error::InconsistentDesign(format!(
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
        (Locator::MaxN(locator), Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new(locator.bins, locator.steps.as_slice());
            Ok(ticker.ticks(nb))
        }
        (Locator::PiMultiple(locator), Scale::Auto | Scale::Linear { .. }) => {
            let ticker = MaxN::new_pi(locator.bins);
            Ok(ticker.ticks(nb))
        }
        (Locator::Log(locator), Scale::Auto) => Ok(LogLocator::new_minor(locator.base).ticks(nb)),
        (Locator::Log(locator), Scale::Log(LogScale { base, .. })) if locator.base == *base => {
            Ok(LogLocator::new_minor(*base).ticks(nb))
        }
        _ => Err(Error::InconsistentDesign(format!(
            "Unsupported locator/scale combination: {:?}/{:?}",
            locator, scale
        ))),
    }
}

#[cfg(feature = "time")]
pub fn locate_datetime(locator: &Locator, tb: axis::TimeBounds) -> Result<Vec<DateTime>, Error> {
    match locator {
        Locator::Auto | Locator::DateTime(DateTimeLocator::Auto) => {
            let span = tb.span();

            // heuristics to pick a locator that will yield ~ 5 to 10 ticks
            let locator = if span > TimeDelta::from_days(5.0 * 365.0) {
                let years = span.seconds() / (10.0 * 365.0 * 86400.0);
                Locator::DateTime(DateTimeLocator::Years((years as u32).max(1)))
            } else if span > TimeDelta::from_days(5.0 * 30.0) {
                let months = span.seconds() / (10.0 * 30.0 * 86400.0);
                Locator::DateTime(DateTimeLocator::Months((months as u32).max(1)))
            } else if span > TimeDelta::from_days(5.0 * 7.0) {
                let weeks = span.seconds() / (10.0 * 7.0 * 86400.0);
                Locator::DateTime(DateTimeLocator::Weeks((weeks as u32).max(1)))
            } else if span > TimeDelta::from_days(5.0) {
                let days = span.seconds() / (10.0 * 86400.0);
                Locator::DateTime(DateTimeLocator::Days((days as u32).max(1)))
            } else if span > TimeDelta::from_hours(5.0) {
                let hours = span.seconds() / (10.0 * 3600.0);
                Locator::DateTime(DateTimeLocator::Hours((hours as u32).max(1)))
            } else if span > TimeDelta::from_minutes(5.0) {
                let minutes = span.seconds() / (10.0 * 60.0);
                Locator::DateTime(DateTimeLocator::Minutes((minutes as u32).max(1)))
            } else if span > TimeDelta::from_seconds(5.0) {
                let seconds = span.seconds() / 10.0;
                Locator::DateTime(DateTimeLocator::Seconds((seconds as u32).max(1)))
            } else {
                let micro = span.seconds() * 1_000_000.0 / 10.0;
                Locator::DateTime(DateTimeLocator::Micros((micro as u32).max(1)))
            };
            locate_datetime(&locator, tb)
        }
        &Locator::DateTime(DateTimeLocator::Years(n)) => {
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
        &Locator::DateTime(DateTimeLocator::Months(n)) => {
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
        &Locator::DateTime(DateTimeLocator::Weeks(n)) => {
            // TODO: scroll back to Monday
            let start = tb.start();
            let td = TimeDelta::from_seconds(7.0 * 24.0 * 3600.0) * n as f64;
            Ok(locate_datetime_even(start, tb, td))
        }
        &Locator::DateTime(DateTimeLocator::Days(n)) => {
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                ..DateTimeComps::epoch()
            };
            let td = TimeDelta::from_seconds(24.0 * 3600.0) * n as f64;
            Ok(locate_datetime_even(start.try_into().unwrap(), tb, td))
        }
        &Locator::DateTime(DateTimeLocator::Hours(n)) => {
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                hour: start.hour,
                ..DateTimeComps::epoch()
            };
            let td = TimeDelta::from_seconds(3600.0) * n as f64;
            Ok(locate_datetime_even(start.try_into().unwrap(), tb, td))
        }
        &Locator::DateTime(DateTimeLocator::Minutes(n)) => {
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                hour: start.hour,
                minute: start.minute,
                ..DateTimeComps::epoch()
            };
            let td = TimeDelta::from_seconds(60.0) * n as f64;
            Ok(locate_datetime_even(start.try_into().unwrap(), tb, td))
        }
        &Locator::DateTime(DateTimeLocator::Seconds(n)) => {
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                hour: start.hour,
                minute: start.minute,
                second: start.second,
                ..DateTimeComps::epoch()
            };
            let td = TimeDelta::from_seconds(1.0) * n as f64;
            Ok(locate_datetime_even(start.try_into().unwrap(), tb, td))
        }
        &Locator::DateTime(DateTimeLocator::Micros(n)) => {
            let start = tb.start().to_comps();
            let start = DateTimeComps {
                year: start.year,
                month: start.month,
                day: start.day,
                hour: start.hour,
                minute: start.minute,
                second: start.second,
                ..DateTimeComps::epoch()
            };
            let td = TimeDelta::from_seconds(1E-6) * n as f64;
            Ok(locate_datetime_even(start.try_into().unwrap(), tb, td))
        }
        _ => Err(Error::InconsistentDesign(format!(
            "Inconsistent ticks locator for time axis: {locator:?}"
        ))),
    }
}

#[cfg(feature = "time")]
fn locate_datetime_even(start: DateTime, tb: axis::TimeBounds, td: TimeDelta) -> Vec<DateTime> {
    // pushing from one tick before start to one tick after end
    let mut res = Vec::new();
    let mut cur = start;
    if cur >= tb.start() {
        cur -= td;
    }
    while cur < tb.end() {
        if cur + td >= tb.start() {
            res.push(cur);
        }
        cur += td;
    }
    res.push(cur);
    res
}

#[cfg(feature = "time")]
fn time_steps(steps: &[u32], secs: f64, multiplier: f64) -> u32 {
    for s in steps {
        let bins = secs / ((*s as f64) * multiplier);
        if bins < 10.0 {
            return *s;
        }
    }
    steps[steps.len() - 1]
}

#[cfg(feature = "time")]
fn locate_timedelta_num(loc: &TimeDeltaLocator, nb: axis::NumBounds) -> Result<Vec<f64>, Error> {
    let step = match loc {
        TimeDeltaLocator::Auto => {
            let secs = nb.span();
            let locator = if secs > 5.0 * 86400.0 {
                let days = secs / (10.0 * 86400.0);
                TimeDeltaLocator::Days((days as u32).max(1))
            } else if secs > 5.0 * 3600.0 {
                let hours = time_steps(&[1, 2, 3, 4, 6, 12, 24], secs, 3600.0);
                TimeDeltaLocator::Hours(hours)
            } else if secs > 5.0 * 60.0 {
                let minutes = time_steps(&[1, 5, 10, 15, 30], secs, 60.0);
                TimeDeltaLocator::Minutes(minutes)
            } else if secs > 5.0 {
                let seconds = time_steps(&[1, 5, 10, 15, 30], secs, 1.0);
                TimeDeltaLocator::Seconds(seconds)
            } else {
                let micros = time_steps(
                    &[
                        1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000,
                        100000, 200000, 500000,
                    ],
                    secs,
                    1E-6,
                );
                TimeDeltaLocator::Seconds(micros)
            };
            return locate_timedelta_num(&locator, nb);
        }
        TimeDeltaLocator::Days(n) if *n > 0 => *n as f64 * 86400.0,
        TimeDeltaLocator::Hours(n) if *n > 0 => *n as f64 * 3600.0,
        TimeDeltaLocator::Minutes(n) if *n > 0 => *n as f64 * 60.0,
        TimeDeltaLocator::Seconds(n) if *n > 0 => *n as f64,
        TimeDeltaLocator::Micros(n) if *n > 0 => *n as f64 * 1E-6,
        _ => {
            return Err(Error::InconsistentDesign(
                "TimeDeltaLocator with null step".into(),
            ));
        }
    };
    let start = (nb.start() / step).floor() * step;
    let end = nb.end();
    Ok(locate_timedelta_even(start, end, step))
}

#[cfg(feature = "time")]
fn locate_timedelta_even(start: f64, end: f64, step: f64) -> Vec<f64> {
    // pushing from one tick before start to one tick after end
    let mut res = Vec::with_capacity((1.0 + (end - start) / step) as usize);
    let mut current = start;
    while current <= end {
        res.push(current);
        current += step;
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
) -> Arc<dyn LabelFormatter> {
    match ticks.formatter() {
        None => Arc::new(NullFormat),
        Some(Formatter::Auto) if scale.is_shared() => Arc::new(NullFormat),
        Some(Formatter::Auto | Formatter::SharedAuto) => {
            auto_label_formatter(ticks.locator(), ab, scale)
        }
        Some(Formatter::Prec(prec)) => Arc::new(PrecLabelFormat(*prec)),
        Some(Formatter::Percent(fmt)) => {
            let prec = fmt
                .decimal_places
                .unwrap_or_else(|| percent_auto_precision(ab));
            Arc::new(PercentLabelFormat(prec))
        }
        #[cfg(feature = "time")]
        Some(Formatter::TimeDelta(tdfmt)) => timedelta_label_formatter(ab, tdfmt),
        #[cfg(feature = "time")]
        _ => todo!(),
    }
}

fn auto_label_formatter(
    locator: &Locator,
    ab: axis::NumBounds,
    scale: &Scale,
) -> Arc<dyn LabelFormatter> {
    match (locator, scale) {
        (Locator::PiMultiple { .. }, _) => Arc::new(PiMultipleLabelFormat { prec: 2 }),
        (Locator::Auto, Scale::Log(LogScale { base, .. })) if *base == 10.0 => {
            Arc::new(SciLabelFormat)
        }
        (Locator::Auto, _) => {
            let max = ab.start().abs().max(ab.end().abs());
            if max >= 10000.0 || max < 0.01 {
                Arc::new(SciLabelFormat)
            } else if max >= 100.0 {
                Arc::new(PrecLabelFormat(0))
            } else if max >= 10.0 {
                Arc::new(PrecLabelFormat(1))
            } else {
                Arc::new(PrecLabelFormat(2))
            }
        }
        _ => todo!(),
    }
}

fn percent_auto_precision(ab: axis::NumBounds) -> usize {
    let span = ab.span();
    if span >= 1.0 {
        0
    } else if span >= 0.1 {
        1
    } else if span >= 0.01 {
        2
    } else {
        3
    }
}

#[cfg(feature = "time")]
pub fn datetime_label_formatter(
    ticks: &Ticks,
    tb: axis::TimeBounds,
    scale: &Scale,
) -> Result<Arc<dyn LabelFormatter>, Error> {
    match ticks.formatter() {
        Some(Formatter::Auto) if scale.is_shared() => Ok(Arc::new(NullFormat)),
        Some(Formatter::Auto | Formatter::SharedAuto) => auto_datetime_label_formatter(tb),
        Some(Formatter::DateTime(DateTimeFormatter::Auto)) => auto_datetime_label_formatter(tb),
        Some(Formatter::DateTime(DateTimeFormatter::DateTime)) => {
            Ok(Arc::new(DateTimeLabelFormat {
                fmt: "%Y-%m-%d %H:%M:%S".to_string(),
            }))
        }
        Some(Formatter::DateTime(DateTimeFormatter::Date)) => Ok(Arc::new(DateTimeLabelFormat {
            fmt: "%Y-%m-%d".to_string(),
        })),
        Some(Formatter::DateTime(DateTimeFormatter::Time)) => Ok(Arc::new(DateTimeLabelFormat {
            fmt: "%H:%M:%S".to_string(),
        })),
        Some(Formatter::DateTime(DateTimeFormatter::Custom(fmt))) => {
            Ok(Arc::new(DateTimeLabelFormat { fmt: fmt.clone() }))
        }
        None => Ok(Arc::new(NullFormat)),
        _ => todo!(),
    }
}

#[cfg(feature = "time")]
fn auto_datetime_label_formatter(tb: axis::TimeBounds) -> Result<Arc<dyn LabelFormatter>, Error> {
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

    Ok(Arc::new(DateTimeLabelFormat {
        fmt: fmt.to_string(),
    }))
}

#[cfg(feature = "time")]
pub fn timedelta_label_formatter(
    nb: axis::NumBounds,
    tdfmt: &TimeDeltaFormatter,
) -> Arc<dyn LabelFormatter> {
    match tdfmt {
        TimeDeltaFormatter::Auto => {
            let fmt = if nb.span() >= 86400.0 {
                "%D %H:%M:%S".to_string()
            } else {
                "%H:%M:%S".to_string()
            };
            Arc::new(TimeDeltaLabelFormat { fmt })
        }
        TimeDeltaFormatter::Custom(fmt) => Arc::new(TimeDeltaLabelFormat { fmt: fmt.clone() }),
    }
}

pub trait LabelFormatter: std::fmt::Debug {
    fn axis_annotation(&self) -> Option<&str> {
        None
    }
    fn format_label(&self, data: data::SampleRef) -> String;
}

impl LabelFormatter for Categories {
    fn format_label(&self, data: data::SampleRef) -> String {
        let cat = data.as_cat().expect("Should be a category");
        self.iter()
            .find(|c| *c == cat)
            .map(str::to_string)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
struct PrecLabelFormat(usize);

impl LabelFormatter for PrecLabelFormat {
    fn format_label(&self, data: data::SampleRef) -> String {
        let data = data.as_num().unwrap();
        format!("{data:.*}", self.0)
    }
}

#[derive(Debug)]
struct SciLabelFormat;

impl LabelFormatter for SciLabelFormat {
    fn format_label(&self, data: data::SampleRef) -> String {
        let data = data.as_num().unwrap();
        format!("{data:.2e}")
    }
}

#[derive(Debug)]
struct PiMultipleLabelFormat {
    prec: usize,
}

impl LabelFormatter for PiMultipleLabelFormat {
    fn axis_annotation(&self) -> Option<&str> {
        Some("\u{00d7} π")
    }
    fn format_label(&self, data: data::SampleRef) -> String {
        let data = data.as_num().unwrap();
        let val = data / PI;
        format!("{val:.*}", self.prec)
    }
}

#[derive(Debug)]
struct PercentLabelFormat(usize);

impl LabelFormatter for PercentLabelFormat {
    fn format_label(&self, data: data::SampleRef) -> String {
        let data = data.as_num().unwrap();
        format!("{:.*}%", self.0, data * 100.0)
    }
}

#[cfg(feature = "time")]
#[derive(Debug)]
struct DateTimeLabelFormat {
    fmt: String,
}

#[cfg(feature = "time")]
impl LabelFormatter for DateTimeLabelFormat {
    fn format_label(&self, data: data::SampleRef) -> String {
        let dt = data.as_time().unwrap();
        format!("{}", dt.fmt_to_string(&self.fmt))
    }
}

#[cfg(feature = "time")]
#[derive(Debug)]
struct TimeDeltaLabelFormat {
    fmt: String,
}

#[cfg(feature = "time")]
impl LabelFormatter for TimeDeltaLabelFormat {
    fn format_label(&self, data: data::SampleRef) -> String {
        match data {
            data::SampleRef::Num(num) => {
                let td = TimeDelta::from_seconds(num);
                td.fmt_to_string(&self.fmt)
            }
            data::SampleRef::TimeDelta(td) => td.fmt_to_string(&self.fmt),
            _ => panic!("data is not compatible with formatter"),
        }
    }
}

#[derive(Debug)]
struct NullFormat;

impl LabelFormatter for NullFormat {
    fn format_label(&self, _: data::SampleRef) -> String {
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
