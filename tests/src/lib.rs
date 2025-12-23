#![cfg(test)]

use eidoplot::{Style, style};

mod harness;
mod pixelmatch;
mod tests;

use harness::{PxlHarness, SvgHarness, TestHarness};

fn bw_theme() -> Style {
    style::Builtin::BlackWhite.to_style()
}

macro_rules! assert_fig_eq_ref {
    (pxl, $fig:expr, $ref_name:expr, $style:expr) => {
        if let Err(err) = $crate::PxlHarness::check_fig_eq_ref($fig, $ref_name, $style) {
            panic!("{}", err);
        }
    };
    (pxl, $fig:expr, $ref_name:expr) => {
        assert_fig_eq_ref!(pxl, $fig, $ref_name, $crate::bw_theme());
    };
    (svg, $fig:expr, $ref_name:expr, $style:expr) => {
        if let Err(err) = $crate::SvgHarness::check_fig_eq_ref($fig, $ref_name, $style) {
            panic!("{}", err);
        }
    };
    (svg, $fig:expr, $ref_name:expr) => {
        assert_fig_eq_ref!(svg, $fig, $ref_name, $crate::bw_theme());
    };

    ($fig:expr, $ref_name:expr, $style:expr) => {
        let mut err = String::new();
        if let Err(e) = $crate::PxlHarness::check_fig_eq_ref($fig, $ref_name, $style) {
            err = e;
        }
        if let Err(e) = $crate::SvgHarness::check_fig_eq_ref($fig, $ref_name, $style) {
            if !err.is_empty() {
                err.push_str("\n\n");
            }
            err.push_str(&e);
        }
        if !err.is_empty() {
            panic!("\n{}\n", err);
        }
    };
    ($fig:expr, $ref_name:expr) => {
        assert_fig_eq_ref!($fig, $ref_name, &$crate::bw_theme());
    };
}

pub(crate) use assert_fig_eq_ref;
