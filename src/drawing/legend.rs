use crate::drawing::Text;
use crate::geom::{Padding, Size};
use crate::style::{defaults, theme};
use crate::text::{self, LineText, fontdb};
use crate::{Style, drawing, geom, des, render, style};

#[derive(Debug, Clone)]
pub enum Shape {
    Line(style::series::Stroke),
    Marker(style::series::Marker),
    Rect(style::series::Fill, Option<style::series::Stroke>),
}

#[derive(Debug, Clone, Copy)]
pub enum ShapeRef<'a> {
    Line(&'a style::series::Stroke),
    Marker(&'a style::series::Marker),
    Rect(&'a style::series::Fill, Option<&'a style::series::Stroke>),
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
    pub font: Option<&'a des::legend::EntryFont>,
    pub shape: ShapeRef<'a>,
}

/// A legend entry, as built during setup phase
#[derive(Debug, Clone)]
struct LegendEntry {
    index: usize,
    shape: Shape,
    text: Text,
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
pub struct LegendBuilder<'a> {
    font: des::legend::EntryFont,
    fill: Option<theme::Fill>,
    border: Option<theme::Stroke>,
    columns: Option<u32>,
    spacing: Size,
    padding: Padding,

    avail_width: f32,
    fontdb: &'a fontdb::Database,
    entries: Vec<LegendEntry>,
}

#[derive(Debug, Clone)]
pub struct Legend {
    fill: Option<theme::Fill>,
    border: Option<theme::Stroke>,
    entries: Vec<LegendEntry>,

    size: geom::Size,
}

impl<'a> LegendBuilder<'a> {
    pub fn from_ir(
        legend: &des::Legend,
        prefers_vertical: bool,
        avail_width: f32,
        fontdb: &'a fontdb::Database,
    ) -> LegendBuilder<'a> {
        let mut columns = legend.columns();
        if columns.is_none() && prefers_vertical {
            columns.replace(1);
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
        let text = Text::from_line_text(&text, &self.fontdb, font.color)?;
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
        let row_height = self.max_entry_height();
        let column_width = self.max_entry_width();
        let columns = self
            .columns
            .unwrap_or_else(|| self.calc_columns(column_width))
            .max(1);
        let mut col = 0;
        let mut x = self.padding.left();
        let mut y = self.padding.top();
        let mut w: f32 = 0.0;
        let mut h: f32 = 0.0;
        for e in &mut self.entries {
            e.x = x;
            e.y = y;
            w = w.max(x + column_width);
            h = h.max(y + row_height);
            if col == columns - 1 {
                col = 0;
                x = self.padding.left();
                y += row_height + self.spacing.height();
            } else {
                col += 1;
                x += column_width + self.spacing.width();
            }
        }
        let sz = geom::Size::new(w + self.padding.right(), h + self.padding.bottom());
        Some(Legend {
            fill: self.fill,
            border: self.border,
            entries: self.entries,
            size: sz,
        })
    }

    fn max_entry_height(&self) -> f32 {
        let mut height = f32::NAN;
        for e in &self.entries {
            height = height.max(e.height());
        }
        height
    }

    fn max_entry_width(&self) -> f32 {
        let mut width = f32::NAN;
        for e in &self.entries {
            width = width.max(e.width());
        }
        width
    }

    fn calc_columns(&self, column_width: f32) -> u32 {
        let avail_width = self.avail_width - self.padding.sum_hor();
        let mut cols = 1;
        let mut width = column_width;
        while (width + column_width + self.spacing.width()) < avail_width {
            width += column_width + self.spacing.width();
            cols += 1;
        }
        cols
    }
}

impl Legend {
    pub fn size(&self) -> geom::Size {
        self.size
    }

    pub fn draw<S>(&self, surface: &mut S, style: &Style, top_left: &geom::Point)
    where
        S: render::Surface,
    {
        let rect = geom::Rect::from_ps(*top_left, self.size);
        if self.fill.is_some() || self.border.is_some() {
            surface.draw_rect(&render::Rect {
                rect,
                fill: self.fill.map(|f| f.as_paint(style)),
                stroke: self.border.as_ref().map(|l| l.as_stroke(style)),
                transform: None,
            });
        }

        for entry in &self.entries {
            entry.draw(surface, style, &rect);
        }
    }
}

impl LegendEntry {
    fn draw<S>(&self, surface: &mut S, style: &Style, rect: &geom::Rect)
    where
        S: render::Surface,
    {
        let rect = geom::Rect::from_xywh(
            rect.left() + self.x,
            rect.top() + self.y,
            self.width(),
            self.height(),
        );

        let shape_sz = defaults::LEGEND_SHAPE_SIZE;
        let shape_rect = geom::Rect::from_ps(
            geom::Point {
                x: rect.left(),
                y: rect.center_y() - shape_sz.height() / 2.0,
            },
            shape_sz,
        );

        let rc = (style, self.index);

        match &self.shape {
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
                surface.draw_path(&line);
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
                surface.draw_path(&path);
            }
            Shape::Rect(fill, line) => {
                let r = geom::Rect::from_ps(
                    geom::Point {
                        x: rect.left(),
                        y: rect.center_y() - shape_sz.height() / 2.0,
                    },
                    shape_sz,
                );
                let rr = render::Rect {
                    rect: r,
                    fill: Some(fill.as_paint(&rc)),
                    stroke: line.as_ref().map(|l| l.as_stroke(&rc)),
                    transform: None,
                };
                surface.draw_rect(&rr);
            }
        };

        let transform = geom::Transform::from_translate(
            rect.left() + shape_sz.width() + defaults::LEGEND_SHAPE_SPACING,
            rect.center_y(),
        );
        self.text.draw(surface, style, Some(&transform));
    }
}
