use crate::{data, style::{color, Color}, text::{self, FontFamily}};

pub const DEFAULT_LABEL_FONT_SIZE: f32 = 12.0;
pub const DEFAULT_LABEL_COLOR: Color = color::BLACK;

#[derive(Debug, Default, Clone)]
pub enum Locator {
    #[default]
    Auto,
    MaxN { bins: u32, steps: Vec<f64> },
    PiMultiple { bins: u32 },
}

impl Locator {
    pub fn ticks(&self, vb: data::ViewBounds) -> Vec<f64> {
        match self {
            Locator::Auto => MaxN::new_auto().ticks(vb),
            Locator::MaxN { bins, steps } => {
                let ticker = MaxN::new(*bins, steps.as_slice());
                ticker.ticks(vb)
            }
            Locator::PiMultiple { bins } => {
                let ticker = MaxN::new_pi(*bins);
                ticker.ticks(vb)
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum Formatter {
    #[default]
    Auto,
    Prec(usize),
}

impl Formatter {
    pub fn label_format(&self, locator: &Locator, dvb: data::ViewBounds) -> Box<dyn LabelFormat> {
        match self {
            Formatter::Auto => Self::auto_label_format(locator, dvb),
            Formatter::Prec(prec) => Box::new(PrecLabelFormat(*prec)),
        }
    }

    fn auto_label_format(locator: &Locator, _dvb: data::ViewBounds) -> Box<dyn LabelFormat> {
        match locator {
            Locator::PiMultiple { .. } => Box::new(PiMultipleLabelFormat{prec: 2}),
            _ => todo!(),
        }
    }
}

pub trait LabelFormat {
    fn axis_annotation(&self) -> Option<&str> {
        None
    }
    fn format_label(&self, data: f64) -> String;
}

struct PrecLabelFormat(usize);

impl LabelFormat for PrecLabelFormat {
    fn format_label(&self, data: f64) -> String {
        format!("{data:.*}", self.0)
    }
}

struct PiMultipleLabelFormat {
    prec: usize,
}

impl LabelFormat for PiMultipleLabelFormat {
    fn axis_annotation(&self) -> Option<&str> {
        Some("\u{00d7} π")
    }
    fn format_label(&self, data: f64) -> String {
        let val = data / PI; 
        format!("{val:.*}", self.prec)
    }
}

#[derive(Debug, Clone)]
pub struct Ticks {
    locator: Locator,
    formatter: Formatter,
    font: text::Font,
    color: Color,
}

impl Default for Ticks {
    fn default() -> Self {
        Ticks {
            locator: Locator::default(),
            formatter: Formatter::default(),
            font: text::Font::new(FontFamily::default(), DEFAULT_LABEL_FONT_SIZE),
            color: DEFAULT_LABEL_COLOR,
        }
    }
}

impl Ticks {
    pub fn with_locator(mut self, locator: Locator) -> Self {
        self.locator = locator;
        self
    }
    pub fn with_formatter(mut self, formatter: Formatter) -> Self {
        self.formatter = formatter;
        self
    }
    pub fn with_font(mut self, font: text::Font) -> Self {
        self.font = font;
        self
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn locator(&self) -> &Locator {
        &self.locator
    }
    pub fn formatter(&self) -> &Formatter {
        &self.formatter
    }
    pub fn font(&self) -> &text::Font {
        &self.font
    }
    pub fn color(&self) -> Color {
        self.color
    }
}

impl From<Locator> for Ticks {
    fn from(value: Locator) -> Self {
        Ticks {
            locator: value,
            ..Default::default()
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

    fn ticks(&self, vb: data::ViewBounds) -> Vec<f64> {
        let target_step = vb.span() / self.bins as f64;

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

        let vmin = (vb.min() / step).floor() * step;

        let edge = MaxNEdge { step };
        let low = edge.largest_le(vb.min() - vmin);
        let high = edge.smallest_ge(vb.max() - vmin);

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
