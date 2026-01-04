#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
/*!
 * # plotive
 * _declarative plotting_. A simple data plotting library written in Rust
 *
 * Plotive separates figure design from data representation and from rendering surfaces.
 *
 * ## Supported figure types
 *  - XY line plots
 *  - Scatter plots
 *  - Histograms
 *  - Bar plots
 *
 * ## Notes about plotive's design
 *
 * The figure design lies in the [`des`] module.
 * This module describes the figure design in a declarative way. It ignores everything about the rendering surfaces.
 * This will allow to bridge easily plotive to other programming languages and to write a compiler for plotive figures.
 *
 * The rendering surfaces implements the [`render::Surface`] trait and are in separate crates.
 * (see `plotive-pxl`, `plotive-svg`, `plotive-iced`)
 * The rendering surfaces themselves ignore everything about figure design and the `des` module.
 * They focus on rendering primitives, like rects and paths. Even text is pre-processed to paths before reaching the surface.
 * This allows to easily add new rendering surfaces to plotive.
 *
 * [`des`] and [`render`] are bridged together by the [`drawing`] module, which exposes very little public API.
 * This module draws a [`des::Figure`] onto a [`render::Surface`] and acts in two phases:
 *  - preparation: [`drawing::Drawing::prepare()`] returns a [`drawing::Figure`], which caches all the layout information,
 *    the text preprocessed as paths, the series data converted to paths, etc.
 *  - drawing: [`drawing::Figure::draw()`] draws the prepared figure onto the surface, using the cached information. Themes
 *    colors are resolved at this stage.
 *
 * The [`drawing::Figure`] has API to update the series with new data, so that dynamic plots can be implemented easily.
 * It also supports zooming and panning operations.
 *
 * ## Example
 *
 * ```no_run
 * use std::f64::consts::PI;
 * use std::sync::Arc;
 *
 * use plotive::{data, des, style};
 * use plotive_iced::{Show, show};
 *
 * fn main() {
 *     let x_axis = des::Axis::new()
 *         .with_title("x".into())
 *         .with_ticks(
 *             des::axis::Ticks::new()
 *                 .with_locator(des::axis::ticks::PiMultipleLocator::default().into()),
 *         )
 *         .with_grid(Default::default());
 *
 *     let y_axis = des::Axis::new()
 *         .with_title("y".into())
 *         .with_ticks(Default::default())
 *         .with_grid(Default::default())
 *         .with_minor_ticks(Default::default())
 *         .with_minor_grid(Default::default());
 *
 *     let series = des::series::Line::new(des::data_src_ref("x"), des::data_src_ref("y"))
 *         .with_name("y=sin(x)")
 *         .into();
 *
 *     let plot = des::Plot::new(vec![series])
 *         .with_x_axis(x_axis)
 *         .with_y_axis(y_axis)
 *         .with_legend(des::plot::LegendPos::InTopRight.into());
 *
 *     let fig = des::Figure::new(plot.into()).with_title("a sine wave".into());
 *
 *     let x: Vec<f64> = (0..=360).map(|t| t as f64 * PI / 180.0).collect();
 *     let y = x.iter().map(|x| x.sin()).collect();
 *
 *     let data_source = data::TableSource::new()
 *         .with_f64_column("x", x)
 *         .with_f64_column("y", y);
 *
 *     fig.show(
 *         Arc::new(data_source),
 *         show::Params {
 *             style: Some(style::Builtin::Light.into()),
 *             ..Default::default()
 *         },
 *     )
 *     .unwrap();
 * }
 * ```
 */
// Plotive is released under the MIT License with the following copyright:
// Copyright (c) 2025 Rémi Thebault

pub mod data;
pub mod des;
pub mod drawing;
pub mod render;
pub mod style;

#[cfg(feature = "dsl")]
pub mod eplt;

#[cfg(feature = "time")]
pub mod time;

pub use drawing::Drawing;
pub use style::Style;

/// Rexports of [`plotive_base::color`]` items
pub mod color {
    pub use plotive_base::color::*;
}
pub use color::{Color, ColorU8, ResolveColor};

#[cfg(feature = "dsl")]
/// Rexports of [`plotive_dsl`]` items
pub mod dsl {
    pub use plotive_dsl::*;
}

/// Rexports of [`plotive_base::geom`]` items
pub mod geom {
    pub use plotive_base::geom::*;
}

/// Rexports of [`plotive_text`]` items
pub mod text {
    pub use plotive_text::*;
}
#[cfg(any(
    feature = "noto-sans",
    feature = "noto-sans-italic",
    feature = "noto-serif",
    feature = "noto-serif-italic",
    feature = "noto-mono"
))]
/// Loads fonts that are bundled with plotive
/// and returns the database.
pub use text::bundled_font_db;
pub use text::fontdb;

#[cfg(feature = "utils")]
pub mod utils {
    //! Utility functions for data generation

    #[cfg(feature = "time")]
    use crate::time::DateTime;

    /// Create a linearly spaced vector of `num` elements between `start` and `end`
    pub fn linspace(start: f64, end: f64, num: usize) -> Vec<f64> {
        let step = (end - start) / (num as f64 - 1.0);
        (0..num).map(|i| start + i as f64 * step).collect()
    }

    /// Create a log-spaced vector of `num` elements between `start` and `end`
    pub fn logspace(start: f64, end: f64, num: usize) -> Vec<f64> {
        let log_start = start.log10();
        let log_end = end.log10();
        let step = (log_end - log_start) / (num as f64 - 1.0);
        (0..num)
            .map(|i| 10f64.powf(log_start + i as f64 * step))
            .collect()
    }

    #[cfg(feature = "time")]
    /// Create a linearly spaced time vector of `num` elements between `start` and `end`
    pub fn timespace(start: DateTime, end: DateTime, num: usize) -> Vec<DateTime> {
        let step = (end - start) / (num as f64 - 1.0);
        let mut result = Vec::with_capacity(num);
        let mut cur = start;
        for _ in 0..num {
            result.push(cur);
            cur += step;
        }
        result
    }
}

/// Module containing missing configuration values
/// Basically we put here all magic values that would require proper parameters
mod missing_params {
    use crate::geom;

    pub const FIG_TITLE_MARGIN: f32 = 12.0;

    pub const PLOT_PADDING: geom::Padding = geom::Padding::Even(0.0);

    pub const AXIS_MARGIN: f32 = 10.0;
    pub const AXIS_TITLE_MARGIN: f32 = 8.0;
    pub const AXIS_ANNOT_MARGIN: f32 = 4.0;
    pub const AXIS_SPINE_WIDTH: f32 = 1.0;

    pub const TICK_SIZE: f32 = 4.0;
    pub const TICK_LABEL_MARGIN: f32 = 4.0;
    pub const MINOR_TICK_LINE_WIDTH: f32 = 0.5;
    pub const MINOR_TICK_SIZE: f32 = 2.0;
}

#[cfg(test)]
pub(crate) mod tests {
    pub trait Near {
        fn near_abs(&self, other: &Self, tol: f64) -> bool;
        fn near_rel(&self, other: &Self, err: f64) -> bool;
    }

    impl Near for f64 {
        fn near_abs(&self, other: &Self, tol: f64) -> bool {
            (self - other).abs() <= tol
        }

        fn near_rel(&self, other: &Self, err: f64) -> bool {
            let diff = (self - other).abs();
            let largest = self.abs().max(other.abs());
            diff <= largest * err
        }
    }

    impl Near for f32 {
        fn near_abs(&self, other: &Self, tol: f64) -> bool {
            (self - other).abs() as f64 <= tol
        }

        fn near_rel(&self, other: &Self, err: f64) -> bool {
            let diff = (self - other).abs() as f64;
            let largest = self.abs().max(other.abs()) as f64;
            diff <= largest * err
        }
    }

    macro_rules! assert_near {
        (abs, $a:expr, $b:expr, $tol:expr) => {
            assert!($a.near_abs(&$b, $tol), "Assertion failed: Values are not close enough.\nValue 1: {:?}\nValue 2: {:?}\nTolerance: {}", $a, $b, $tol);
        };
        (abs, $a:expr, $b:expr) => {
            assert_near!(abs, $a, $b, 1e-8);
        };
        (rel, $a:expr, $b:expr, $err:expr) => {
            assert!($a.near_rel(&$b, $err), "Assertion failed: Values are not close enough.\nValue 1: {:?}\nValue 2: {:?}\nRelative error: {}", $a, $b, $err);
        };
        (rel, $a:expr, $b:expr) => {
            assert_near!(rel, $a, $b, 1e-8);
        };
    }

    pub(crate) use assert_near;

    #[test]
    fn test_close_to() {
        let a = 1.0;
        let b = 1.0 + 1e-9;
        assert_near!(abs, a, b);
        assert!(!a.near_abs(&b, 1e-10));
        assert_near!(rel, a, b);
        assert!(!a.near_rel(&b, 1e-10));
    }
}
