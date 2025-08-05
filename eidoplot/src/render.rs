use crate::{geom, style, text};

#[derive(Debug, Clone)]
pub struct Rect {
    pub rect: geom::Rect,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
    pub transform: Option<geom::Transform>,
}

impl Rect {
    pub fn new(rect: geom::Rect) -> Self {
        Rect {
            rect,
            fill: None,
            stroke: None,
            transform: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: geom::Path,
    pub fill: Option<style::Fill>,
    pub stroke: Option<style::Line>,
    pub transform: Option<geom::Transform>,
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub path: geom::Path,
    pub transform: Option<geom::Transform>,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Center
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextBaseline {
    Base,
    Center,
    Hanging,
}

impl Default for TextBaseline {
    fn default() -> Self {
        TextBaseline::Base
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextAnchor {
    pub pos: geom::Point,
    pub align: TextAlign,
    pub baseline: TextBaseline,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: text::Text,
    pub fill: style::Fill,
    pub anchor: TextAnchor,
    pub transform: Option<geom::Transform>,
}
