use crate::{geom, style};

pub use tiny_skia_path::{Path, PathBuilder, PathSegment};

pub struct Rect {
    pub rect: geom::Rect,
    pub fill: Option<style::Fill>,
    pub outline: Option<style::Line>,
}

impl Rect {
    pub fn new(rect: geom::Rect) -> Self {
        Rect {
            rect,
            fill: None,
            outline: None,
        }
    }
}
