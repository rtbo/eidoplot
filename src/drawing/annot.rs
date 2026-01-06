use std::f32;

use super::Ctx;
use crate::drawing::axis::Axis;
use crate::drawing::plot::{Axes, Orientation};
use crate::drawing::{Text, marker};
use crate::des::annot::{Anchor, Direction, Pos, ZPos};
use crate::des::{self};
use crate::style::{self, theme};
use crate::{Style, data, geom, render, text};

#[derive(Debug, Clone)]
pub(super) enum Annot {
    Line(des::annot::Line),
    Arrow(des::annot::Arrow),
    Marker(des::annot::Marker),
    Label(Label),
}

#[derive(Debug, Clone)]
pub(super) struct Label {
    text: Text,
    frame: (Option<theme::Fill>, Option<theme::Line>),
    angle: f32,
    pos: Pos,
}

impl<D> Ctx<'_, D>
where
    D: data::Source + ?Sized,
{
    pub fn setup_annot(&self, annot: &des::Annotation, axes: &Axes) -> Result<Annot, super::Error> {
        let mut annot = match annot {
            des::Annotation::Line(line) => Annot::Line(line.clone()),
            des::Annotation::Arrow(arrow) => Annot::Arrow(arrow.clone()),
            des::Annotation::Marker(marker) => Annot::Marker(marker.clone()),
            des::Annotation::Label(label) => {
                let (align, ver_align) = match label.anchor {
                    Anchor::TopLeft => (text::line::Align::Left, text::line::VerAlign::Top),
                    Anchor::TopCenter => (text::line::Align::Center, text::line::VerAlign::Top),
                    Anchor::TopRight => (text::line::Align::Right, text::line::VerAlign::Top),
                    Anchor::CenterRight => (text::line::Align::Right, text::line::VerAlign::Middle),
                    Anchor::BottomRight => (text::line::Align::Right, text::line::VerAlign::Bottom),
                    Anchor::BottomCenter => {
                        (text::line::Align::Center, text::line::VerAlign::Bottom)
                    }
                    Anchor::BottomLeft => (text::line::Align::Left, text::line::VerAlign::Bottom),
                    Anchor::CenterLeft => (text::line::Align::Left, text::line::VerAlign::Middle),
                    Anchor::Center => (text::line::Align::Center, text::line::VerAlign::Middle),
                };
                let line_text = text::LineText::new(
                    label.text.clone(),
                    (align, ver_align),
                    label.font_size,
                    label.font.clone(),
                    &self.fontdb,
                )?;
                let text = Text::from_line_text(&line_text, &self.fontdb, label.color)?;
                Annot::Label(Label {
                    text,
                    frame: label.frame.clone(),
                    angle: label.angle,
                    pos: label.pos.clone(),
                })
            }
        };

        // Resolve axis reference to index, to ensure no error can happen later during drawing
        let x_axis = axes
            .or_find_idx(Orientation::X, &annot.pos().x_axis)?
            .ok_or_else(|| {
                super::Error::UnknownAxisRef(annot.pos().x_axis.clone())
            })?;
        let y_axis = axes
            .or_find_idx(Orientation::Y, &annot.pos().y_axis)?
            .ok_or_else(|| {
                super::Error::UnknownAxisRef(annot.pos().y_axis.clone())
            })?;
        annot.pos_mut().x_axis = des::axis::Ref::Idx(x_axis);
        annot.pos_mut().y_axis = des::axis::Ref::Idx(y_axis);

        Ok(annot)
    }
}

impl Annot {
    fn pos(&self) -> &Pos {
        match self {
            Annot::Line(line) => &line.pos,
            Annot::Arrow(arrow) => &arrow.pos,
            Annot::Marker(marker) => &marker.pos,
            Annot::Label(label) => &label.pos,
        }
    }

    fn pos_mut(&mut self) -> &mut Pos {
        match self {
            Annot::Line(line) => &mut line.pos,
            Annot::Arrow(arrow) => &mut arrow.pos,
            Annot::Marker(marker) => &mut marker.pos,
            Annot::Label(label) => &mut label.pos,
        }
    }

    pub fn zpos(&self) -> ZPos {
        self.pos().zpos
    }

    pub fn draw<S>(
        &self,
        surface: &mut S,
        style: &style::Style,
        axes: &Axes,
        plot_rect: &geom::Rect,
    ) where
        S: render::Surface,
    {
        let x_axis = axes
            .or_find(Orientation::X, &self.pos().x_axis)
            .unwrap()
            .unwrap();
        let y_axis = axes
            .or_find(Orientation::Y, &self.pos().y_axis)
            .unwrap()
            .unwrap();
        match self {
            Annot::Line(line) => {
                self.draw_annot_line(surface, style, line, &x_axis, &y_axis, plot_rect);
            }
            Annot::Arrow(arrow) => {
                self.draw_annot_arrow(surface, style, arrow, &x_axis, &y_axis, plot_rect);
            }
            Annot::Marker(marker) => {
                self.draw_annot_marker(surface, style, marker, x_axis, y_axis, plot_rect);
            }
            Annot::Label(label) => {
                self.draw_annot_label(surface, style, label, &x_axis, &y_axis, plot_rect);
            }
        }
    }

    fn draw_annot_line<S>(
        &self,
        surface: &mut S,
        style: &Style,
        line: &des::annot::Line,
        x_axis: &Axis,
        y_axis: &Axis,
        plot_rect: &geom::Rect,
    ) where
        S: render::Surface,
    {
        let (x, y) = (line.pos.x, line.pos.y);
        let (p1, p2) = match line.direction {
            Direction::Horizontal => {
                let y = y_axis.coord_map().map_coord_num(y);
                let p1 = geom::Point {
                    x: plot_rect.left(),
                    y,
                };
                let p2 = geom::Point {
                    x: plot_rect.right(),
                    y,
                };
                (p1, p2)
            }
            Direction::Vertical => {
                let x = x_axis.coord_map().map_coord_num(x);
                let p1 = geom::Point {
                    x,
                    y: plot_rect.top(),
                };
                let p2 = geom::Point {
                    x,
                    y: plot_rect.bottom(),
                };
                (p1, p2)
            }
            Direction::Slope(slope) => {
                // FIXME: raise error if either X or Y is logarithmic
                let x1 = x_axis.coord_map().map_coord_num(x);
                let y1 = y_axis.coord_map().map_coord_num(y);
                let x2 = x1 + 100.0;
                let y2 = y1 + 100.0 * slope;
                let p1 = geom::Point { x: x1, y: y1 };
                let p2 = geom::Point { x: x2, y: y2 };
                (p1, p2)
            }
            Direction::SecondPoint(x2, y2) => {
                let x1 = x_axis.coord_map().map_coord_num(x);
                let y1 = y_axis.coord_map().map_coord_num(y);
                let x2 = x_axis.coord_map().map_coord_num(x2);
                let y2 = y_axis.coord_map().map_coord_num(y2);
                let p1 = geom::Point { x: x1, y: y1 };
                let p2 = geom::Point { x: x2, y: y2 };
                (p1, p2)
            }
        };

        let p1 = geom::Point {
            x: p1.x + plot_rect.left(),
            y: plot_rect.bottom() - p1.y,
        };
        let p2 = geom::Point {
            x: p2.x + plot_rect.left(),
            y: plot_rect.bottom() - p2.y,
        };

        let points = plot_rect_intersections(plot_rect, &p1, &p2);
        if let Some([p1, p2]) = points {
            let mut path = geom::PathBuilder::with_capacity(2, 2);
            path.move_to(p1.x, p1.y);
            path.line_to(p2.x, p2.y);
            let path = path.finish().expect("Should be a valid path");
            let path = render::Path {
                path: &path,
                fill: None,
                stroke: Some(line.line.as_stroke(style)),
                transform: None,
            };
            surface.draw_path(&path);
        }
    }

    fn draw_annot_arrow<S>(
        &self,
        surface: &mut S,
        style: &Style,
        arrow: &des::annot::Arrow,
        x_axis: &Axis,
        y_axis: &Axis,
        plot_rect: &geom::Rect,
    ) where
        S: render::Surface,
    {
        let target_x = x_axis.coord_map().map_coord_num(arrow.pos.x);
        let target_y = y_axis.coord_map().map_coord_num(arrow.pos.y);
        let len = (arrow.dx.powi(2) + arrow.dy.powi(2)).sqrt();
        let mut builder = geom::PathBuilder::with_capacity(5, 5);
        builder.move_to(0.0, 0.0);
        builder.line_to(0.0, len);
        builder.move_to(-arrow.head_size / 2.0, arrow.head_size);
        builder.line_to(0.0, 0.0);
        builder.line_to(arrow.head_size / 2.0, arrow.head_size);
        let path = builder.finish().expect("Should be a valid path");
        let angle = (arrow.dy.atan2(arrow.dx) + f32::consts::FRAC_PI_2) * 180.0 / f32::consts::PI;
        let transform = geom::Transform::from_translate(
            plot_rect.left() + target_x,
            plot_rect.bottom() - target_y,
        )
        .pre_rotate(angle);
        let rpath = render::Path {
            path: &path,
            fill: None,
            stroke: Some(arrow.line.as_stroke(style)),
            transform: Some(&transform),
        };
        surface.draw_path(&rpath);
    }

    fn draw_annot_marker<S>(
        &self,
        surface: &mut S,
        style: &Style,
        marker: &des::annot::Marker,
        x_axis: &Axis,
        y_axis: &Axis,
        plot_rect: &geom::Rect,
    ) where
        S: render::Surface,
    {
        let x = x_axis.coord_map().map_coord_num(marker.pos.x);
        let y = y_axis.coord_map().map_coord_num(marker.pos.y);
        let path = marker::marker_path(&marker.marker);

        let transform =
            geom::Transform::from_translate(plot_rect.left() + x, plot_rect.bottom() - y);
        let rpath = render::Path {
            path: &path,
            fill: marker.marker.fill.as_ref().map(|f| f.as_paint(style)),
            stroke: marker.marker.stroke.as_ref().map(|l| l.as_stroke(style)),
            transform: Some(&transform),
        };
        surface.draw_path(&rpath);
    }

    fn draw_annot_label<S>(
        &self,
        surface: &mut S,
        style: &Style,
        label: &Label,
        x_axis: &Axis,
        y_axis: &Axis,
        plot_rect: &geom::Rect,
    ) where
        S: render::Surface,
    {
        let x = x_axis.coord_map().map_coord_num(label.pos.x);
        let y = y_axis.coord_map().map_coord_num(label.pos.y);

        let transform =
            geom::Transform::from_translate(plot_rect.left() + x, plot_rect.bottom() - y)
                .pre_rotate(-label.angle);

        if label.frame.0.is_some() || label.frame.1.is_some() {
            let bounds = label.text.bbox.expect("Text bbox should be computed");
            let rect =
                geom::Rect::from_xywh(bounds.x(), bounds.y(), bounds.width(), bounds.height());
            let rrect = render::Rect {
                rect,
                fill: label.frame.0.as_ref().map(|f| f.as_paint(style)),
                stroke: label.frame.1.as_ref().map(|l| l.as_stroke(style)),
                transform: Some(&transform),
            };
            surface.draw_rect(&rrect);
        }

        label.text.draw(surface, style, Some(&transform));
    }
}

fn plot_rect_intersections(
    plot_rect: &geom::Rect,
    p1: &geom::Point,
    p2: &geom::Point,
) -> Option<[geom::Point; 2]> {
    let mut intersections: [Option<geom::Point>; 4] = [None; 4];

    // Parametric equation of the line: p1 + t * (p2 - p1)
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    // Function to calculate Y for given X (if dx != 0)
    let y_for_x = |x: f32| -> f32 {
        if dx == 0.0 {
            p1.y // vertical line
        } else {
            let t = (x - p1.x) / dx;
            p1.y + t * dy
        }
    };

    // Function to calculate X for given Y (if dx != 0)
    let x_for_y = |y: f32| -> f32 {
        if dy == 0.0 {
            p1.x // horizontal line
        } else {
            let t = (y - p1.y) / dy;
            p1.x + t * dx
        }
    };

    let mut idx = 0;

    // Intersection with vertical edges (left and right)
    if dx != 0.0 {
        for &x in &[plot_rect.x(), plot_rect.x() + plot_rect.width()] {
            let y = y_for_x(x);
            if y >= plot_rect.y() && y <= plot_rect.y() + plot_rect.height() {
                intersections[idx] = Some(geom::Point { x, y });
                idx += 1;
            }
        }
    }

    // Intersection with horizontal edges (top and bottom)
    if dy != 0.0 {
        for &y in &[plot_rect.y(), plot_rect.y() + plot_rect.height()] {
            let x = x_for_y(y);
            if x >= plot_rect.x() && x <= plot_rect.x() + plot_rect.width() {
                intersections[idx] = Some(geom::Point { x, y });
                idx += 1;
            }
        }
    }

    // We return result only if we have two points
    if idx == 2 {
        Some([intersections[0].unwrap(), intersections[1].unwrap()])
    } else {
        None
    }
}
