use std::num::NonZeroU32;
use std::sync::Arc;

use eidoplot_text as text;
use text::{TextLayout, fontdb};

use crate::drawing::{self, Ctx, SurfWrapper};
use crate::render::{self, Surface as _};
use crate::style::defaults;
use crate::{geom, ir, style};

pub enum Shape {
    Line(style::Line),
    Marker(style::Marker),
    Rect(style::Fill, Option<style::Line>),
}

/// trait implemented by series, or any other item that
/// has to populate the legend
pub trait Entry {
    fn label(&self) -> &str;
    fn font(&self) -> Option<&ir::legend::EntryFont>;
    fn shape(&self) -> Shape;
}

struct LegendEntry {
    shape: Shape,
    text: TextLayout,
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
    font: ir::legend::EntryFont,
    fill: Option<style::Fill>,
    border: Option<style::Line>,
    label_fill: style::Fill,
    columns: Option<NonZeroU32>,
    spacing: f32,
    padding: f32,

    avail_width: f32,
    fontdb: Arc<fontdb::Database>,
    entries: Vec<LegendEntry>,

    size: Option<geom::Size>,
}

impl Legend {
    pub fn from_ir(legend: &ir::Legend, prefers_vertical: bool, avail_width: f32, fontdb: Arc<fontdb::Database>) -> Legend {
        let mut columns = legend.columns();
        if columns.is_none() && prefers_vertical {
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

    pub fn add_entry<E>(&mut self, entry: &E) -> Result<(), drawing::Error>
    where
        E: Entry,
    {
        let shape = entry.shape();
        let label = entry.label();
        let entry_font = entry.font();
        let font = entry_font.unwrap_or(&self.font);
        let opts = text::layout::Options {
            hor_align: text::layout::HorAlign::Start,
            ver_align: text::layout::LineVerAlign::Middle.into(),
            ..Default::default()
        };
        let text = text::shape_and_layout_str(label, &font.font, &self.fontdb, font.size, &opts)?;
        let label_width = text.bbox().width();
        let label_height = text.bbox().height();
        self.entries.push(LegendEntry {
            shape,
            text,
            label_width,
            label_height,
            x: f32::NAN,
            y: f32::NAN,
        });
        Ok(())
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
        let sz = geom::Size::new(
            w + self.padding,
            h + self.padding,
        );
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

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_legend<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        legend: &Legend,
        top_left: &geom::Point,
    ) -> Result<(), render::Error> 
    where T: style::Theme,
    {
        let rect = geom::Rect::from_ps(*top_left, legend.size.unwrap());
        if legend.fill.is_some() || legend.border.is_some() {
            self.draw_rect(&render::Rect {
                rect,
                fill: legend.fill.map(|f| f.as_paint(ctx.theme())),
                stroke: legend.border.as_ref().map(|l| l.as_stroke(ctx.theme())),
                transform: None,
            })?;
        }

        for entry in &legend.entries {
            self.draw_legend_entry(ctx, entry, &rect, legend.label_fill)?;
        }

        Ok(())
    }

    fn draw_legend_entry<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        entry: &LegendEntry,
        rect: &geom::Rect,
        label_fill: style::Fill,
    ) -> Result<(), render::Error> 
    where T: style::Theme,
    {
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

        match &entry.shape {
            Shape::Line(line) => {
                let mut path = geom::PathBuilder::new();
                path.move_to(shape_rect.left(), shape_rect.center_y());
                path.line_to(shape_rect.right(), rect.center_y());
                let path = path.finish().expect("Should be a valid path");

                let line = render::Path {
                    path: &path,
                    fill: None,
                    stroke: Some(line.as_stroke(ctx.theme())),
                    transform: None,
                };
                self.draw_path(&line)?;
            }
            Shape::Marker(marker) => {
                let path = crate::drawing::marker::marker_path(&marker);
                let transform = geom::Transform::from_translate(
                    shape_rect.center_x(),
                    shape_rect.center_y(),
                );

                let path = render::Path {
                    path: &path,
                    fill: marker.fill.as_ref().map(|f| f.as_paint(ctx.theme())),
                    stroke: marker.stroke.as_ref().map(|s| s.as_stroke(ctx.theme())),
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
                    fill: Some(fill.as_paint(ctx.theme())),
                    stroke: line.as_ref().map(|l| l.as_stroke(ctx.theme())),
                    transform: None,
                };
                self.draw_rect(&rr)?;
            }
        };

        let pos = geom::Point::new(
            rect.left() + shape_sz.width() + defaults::LEGEND_SHAPE_SPACING,
            rect.center_y(),
        );
        let text = render::TextLayout {
            layout: &entry.text,
            fill: label_fill.as_paint(ctx.theme()),
            transform: Some(&pos.translation()),
        };
        self.draw_text_layout(&text)?;

        Ok(())
    }
}
