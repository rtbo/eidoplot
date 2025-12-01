use std::num::NonZeroU32;
use std::sync::Arc;

use crate::drawing::{self, Ctx, SurfWrapper};
use crate::render::{self, Surface as _};
use crate::style::{Color as _, defaults, theme};
use crate::{geom, ir, style};
use crate::text::{self, fontdb, LineText};

#[derive(Debug, Clone)]
pub enum Shape {
    Line(style::series::Line),
    Marker(style::series::Marker),
    Rect(style::series::Fill, Option<style::series::Line>),
}

#[derive(Debug, Clone, Copy)]
pub enum ShapeRef<'a> {
    Line(&'a style::series::Line),
    Marker(&'a style::series::Marker),
    Rect(&'a style::series::Fill, Option<&'a style::series::Line>),
}

impl ShapeRef<'_> {
    pub fn to_shape(&self) -> Shape {
        match self {
            &ShapeRef::Line(line) => Shape::Line(line.clone()),
            &ShapeRef::Marker(marker) => Shape::Marker(marker.clone()),
            &ShapeRef::Rect(fill, line) => Shape::Rect(fill.clone(), line.cloned()),
        }
    }
}

/// A legend entry, used to populate the legend
#[derive(Debug, Clone)]
pub struct Entry<'a> {
    pub label: &'a str,
    pub font: Option<&'a ir::legend::EntryFont>,
    pub shape: ShapeRef<'a>,
}

/// A legend entry, as built during setup phase
#[derive(Debug, Clone)]
struct LegendEntry {
    index: usize,
    shape: Shape,
    text: LineText,
    x: f32,
    y: f32,
}

impl LegendEntry {
    fn width(&self) -> f32 {
        self.text.width() + defaults::LEGEND_SHAPE_SPACING + defaults::LEGEND_SHAPE_SIZE.width()
    }

    fn height(&self) -> f32 {
        self.text.height().max(defaults::LEGEND_SHAPE_SIZE.height())
    }
}

#[derive(Debug)]
pub struct LegendBuilder {
    font: ir::legend::EntryFont,
    fill: Option<theme::Fill>,
    border: Option<theme::Line>,
    columns: Option<NonZeroU32>,
    spacing: f32,
    padding: f32,

    avail_width: f32,
    fontdb: Arc<fontdb::Database>,
    entries: Vec<LegendEntry>,
}

#[derive(Debug, Clone)]
pub struct Legend {
    font: ir::legend::EntryFont,
    fill: Option<theme::Fill>,
    border: Option<theme::Line>,
    entries: Vec<LegendEntry>,

    size: geom::Size,
}

impl LegendBuilder {
    pub fn from_ir(
        legend: &ir::Legend,
        prefers_vertical: bool,
        avail_width: f32,
        fontdb: Arc<fontdb::Database>,
    ) -> LegendBuilder {
        let mut columns = legend.columns();
        if columns.is_none() && prefers_vertical {
            columns.replace(NonZeroU32::new(1).unwrap());
        }
        LegendBuilder {
            font: legend.font().clone(),
            fill: legend.fill().cloned(),
            border: legend.border().cloned(),
            columns,
            spacing: legend.spacing(),
            padding: legend.padding(),

            avail_width: avail_width,
            fontdb,
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, index: usize, entry: Entry) -> Result<(), drawing::Error> {
        let shape = entry.shape.to_shape();
        let font = entry.font.unwrap_or(&self.font);
        let align = (
            text::line::Align::Start,
            text::line::VerAlign::Middle.into(),
        );
        let text = LineText::new(
            entry.label.to_string(),
            align,
            font.size,
            font.font.clone(),
            &self.fontdb,
        )?;
        self.entries.push(LegendEntry {
            index,
            shape,
            text,
            x: f32::NAN,
            y: f32::NAN,
        });
        Ok(())
    }

    pub fn layout(mut self) -> Option<Legend> {
        if self.entries.is_empty() {
            return None;
        }
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
        Some(Legend {
            font: self.font,
            fill: self.fill,
            border: self.border,
            entries: self.entries,
            size: sz,
        })
    }

    fn max_label_width(&self) -> f32 {
        let mut width = f32::NAN;
        for e in &self.entries {
            width = width.max(e.text.width());
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

impl Legend {
    pub fn size(&self) -> geom::Size {
        self.size
    }
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_legend<D>(
        &mut self,
        ctx: &Ctx<D>,
        legend: &Legend,
        top_left: &geom::Point,
    ) -> Result<(), render::Error> {
        let rect = geom::Rect::from_ps(*top_left, legend.size);
        if legend.fill.is_some() || legend.border.is_some() {
            self.draw_rect(&render::Rect {
                rect,
                fill: legend.fill.map(|f| f.as_paint(ctx.theme())),
                stroke: legend.border.as_ref().map(|l| l.as_stroke(ctx.theme())),
                transform: None,
            })?;
        }

        for entry in &legend.entries {
            self.draw_legend_entry(ctx, entry, &rect, legend.font.color)?;
        }

        Ok(())
    }

    fn draw_legend_entry<D>(
        &mut self,
        ctx: &Ctx<D>,
        entry: &LegendEntry,
        rect: &geom::Rect,
        label_color: theme::Color,
    ) -> Result<(), render::Error> {
        let rect = geom::Rect::from_xywh(
            rect.left() + entry.x,
            rect.top() + entry.y,
            entry.width(),
            entry.height(),
        );

        let shape_sz = defaults::LEGEND_SHAPE_SIZE;
        let shape_rect = geom::Rect::from_ps(
            geom::Point::new(rect.left(), rect.center_y() - shape_sz.height() / 2.0),
            shape_sz,
        );

        let rc = (ctx.theme().palette(), entry.index);

        match &entry.shape {
            Shape::Line(line) => {
                let mut path = geom::PathBuilder::new();
                path.move_to(shape_rect.left(), shape_rect.center_y());
                path.line_to(shape_rect.right(), rect.center_y());
                let path = path.finish().expect("Should be a valid path");

                let line = render::Path {
                    path: &path,
                    fill: None,
                    stroke: Some(line.as_stroke(&rc)),
                    transform: None,
                };
                self.draw_path(&line)?;
            }
            Shape::Marker(marker) => {
                let path = crate::drawing::marker::marker_path(&marker);
                let transform =
                    geom::Transform::from_translate(shape_rect.center_x(), shape_rect.center_y());

                let path = render::Path {
                    path: &path,
                    fill: marker.fill.as_ref().map(|f| f.as_paint(&rc)),
                    stroke: marker.stroke.as_ref().map(|s| s.as_stroke(&rc)),
                    transform: Some(&transform),
                };
                self.draw_path(&path)?;
            }
            Shape::Rect(fill, line) => {
                let r = geom::Rect::from_ps(
                    geom::Point::new(rect.left(), rect.center_y() - shape_sz.height() / 2.0),
                    shape_sz,
                );
                let rr = render::Rect {
                    rect: r,
                    fill: Some(fill.as_paint(&rc)),
                    stroke: line.as_ref().map(|l| l.as_stroke(&rc)),
                    transform: None,
                };
                self.draw_rect(&rr)?;
            }
        };

        let pos = geom::Point::new(
            rect.left() + shape_sz.width() + defaults::LEGEND_SHAPE_SPACING,
            rect.center_y(),
        );
        let transform = pos.translation();
        let rtext = render::LineText {
            text: &entry.text,
            fill: label_color.resolve(ctx.theme()).into(),
            transform,
        };
        self.draw_line_text(&rtext)?;

        Ok(())
    }
}
