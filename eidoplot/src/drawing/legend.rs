use std::num::NonZeroU32;

use crate::drawing::{Ctx, FontDb};
use crate::render::Surface;
use crate::style::defaults;
use crate::{geom, ir, render, style};

pub enum Shape {
    Line(style::Line),
    Rect(style::Fill, Option<style::Line>),
}

/// trait implemented by series, or any other item that
/// has to populate the legend
pub trait Entry {
    fn label(&self) -> &str;
    fn font(&self) -> Option<&style::Font>;
    fn shape(&self) -> Shape;
}

struct LegendEntry {
    shape: Shape,
    label: String,
    font: Option<style::Font>,
    label_width: f32,
    label_height: f32,
    x: f32,
    y: f32,
}

impl LegendEntry {
    fn width(&self) -> f32 {
        self.label_width + defaults::LEGEND_SHAPE_SPACING + defaults::LEGEND_SHAPE_SIZE.width()
    }

    fn height(&self) -> f32 {
        self.label_height.max(defaults::LEGEND_SHAPE_SIZE.height())
    }
}

pub struct Legend {
    font: style::Font,
    fill: Option<style::Fill>,
    border: Option<style::Line>,
    label_fill: style::Fill,
    columns: Option<NonZeroU32>,
    spacing: f32,
    padding: f32,

    avail_width: f32,
    fontdb: FontDb,
    entries: Vec<LegendEntry>,

    size: Option<geom::Size>,
}

impl Legend {
    pub fn from_ir(legend: &ir::Legend, avail_width: f32, fontdb: FontDb) -> Legend {
        let mut columns = legend.columns();
        if columns.is_none() && legend.pos().prefers_vertical() {
            columns.replace(NonZeroU32::new(1).unwrap());
        }
        Legend {
            font: legend.font().clone(),
            fill: legend.fill().cloned(),
            border: legend.border().cloned(),
            label_fill: *legend.label_fill(),
            columns,
            spacing: legend.spacing(),
            padding: legend.padding(),

            avail_width: avail_width,
            fontdb,
            entries: Vec::new(),

            size: None,
        }
    }

    pub fn add_entry<E>(&mut self, entry: &E)
    where
        E: Entry,
    {
        let shape = entry.shape();
        let label = entry.label().to_string();
        let font = entry.font().unwrap_or(&self.font);
        let label_width = self.fontdb.label_width(&label, font);
        let label_height = font.size();
        self.entries.push(LegendEntry {
            shape,
            label,
            font: entry.font().cloned(),
            label_width,
            label_height,
            x: f32::NAN,
            y: f32::NAN,
        });
    }

    pub fn layout(&mut self) -> geom::Size {
        let column_width = self.max_label_width();
        let columns = self
            .columns
            .map(|c| c.get())
            .unwrap_or_else(|| self.calc_columns(column_width))
            .max(1);
        let mut col = 0;
        let mut x = self.padding;
        let mut y = self.padding;
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        for e in &mut self.entries {
            e.x = x;
            e.y = y;
            w = w.max(x + e.width());
            h = h.max(y + e.height());
            if col == columns - 1 {
                col = 0;
                x = self.padding;
                y += e.height() + self.spacing;
            } else {
                col += 1;
                x += e.width() + self.spacing;
            }
        }
        let sz = geom::Size::new(w + self.padding, h + self.padding);
        self.size.replace(sz);
        sz
    }

    fn max_label_width(&self) -> f32 {
        let mut width = f32::NAN;
        for e in &self.entries {
            width = width.max(e.label_width);
        }
        width
    }

    fn calc_columns(&self, column_width: f32) -> u32 {
        let avail_width = self.avail_width - 2.0 * self.padding;
        let mut cols = 1;
        let mut width = column_width;
        while (width + column_width + self.spacing) < avail_width {
            width += column_width + self.spacing;
            cols += 1;
        }
        cols
    }
}

impl<'a, S> Ctx<'a, S>
where
    S: render::Surface,
{
    pub fn draw_legend(&mut self, legend: &Legend, top_left: &geom::Point) -> Result<(), S::Error> {
        let rect = geom::Rect::from_ps(*top_left, legend.size.unwrap());
        if legend.fill.is_some() || legend.border.is_some() {
            self.draw_rect(&render::Rect {
                rect,
                fill: legend.fill,
                stroke: legend.border,
                transform: None,
            })?;
        }

        for entry in &legend.entries {
            self.draw_legend_entry(entry, &rect, &legend.font, legend.label_fill)?;
        }

        Ok(())
    }

    fn draw_legend_entry(&mut self, entry: &LegendEntry, rect: &geom::Rect, font: &style::Font, label_fill: style::Fill) -> Result<(), S::Error> {
        let rect = geom::Rect::from_xywh(
            rect.left() + entry.x,
            rect.top() + entry.y,
            entry.width(),
            entry.height(),
        );

        let shape_sz = defaults::LEGEND_SHAPE_SIZE;

        match entry.shape {
            Shape::Line(line) => {
                let mut path = geom::PathBuilder::new();
                    path.move_to(rect.left(), rect.center_y());
                    path.line_to(rect.left() + shape_sz.width(), rect.center_y());
                    let path = path.finish().expect("Should be a valid path");

                let line = render::Path {
                    path: &path,
                    fill: None,
                    stroke: Some(line),
                    transform: None,
                };
                self.draw_path(&line)?;
            },
            Shape::Rect(fill, line) => {
                let r = geom::Rect::from_ps(geom::Point::new(rect.left(), rect.center_y() - shape_sz.height() / 2.0), shape_sz);
                let rr = render::Rect {
                    rect: r,
                    fill: Some(fill),
                    stroke: line,
                    transform: None,
                };
                self.draw_rect(&rr)?;
            }
        };

        let pos = geom::Point::new(rect.left() + shape_sz.width() + defaults::LEGEND_SHAPE_SPACING, rect.center_y());
        let anchor = render::TextAnchor {
            pos,
            align: render::TextAlign::Start,
            baseline: render::TextBaseline::Center,
        };
        let text = render::Text {
            text: &entry.label,
            font: entry.font.as_ref().unwrap_or(font),
            anchor,
            fill: label_fill,
            transform: None,
        };
        self.draw_text(&text)?;

        Ok(())
    }
}
