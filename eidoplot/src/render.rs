use crate::{geom, style};


pub struct Rect {
    pub rect: geom::Rect,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
}

impl Rect {
    pub fn new(rect: geom::Rect) -> Self {
        Rect {
            rect,
            fill: None,
            stroke: None,
        }
    }
}

pub struct Path {
    pub path: geom::Path,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
}
