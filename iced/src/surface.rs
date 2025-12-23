use eidoplot::{geom, render};
use iced::Rectangle;
use iced::advanced::graphics::geometry::{self, frame};

#[derive(Debug)]
pub struct IcedSurface<Frame> {
    frame: Frame,
    clip_stack: Vec<Frame>,
    bounds: Rectangle,
    transform: geom::Transform,
}

impl<Frame> IcedSurface<Frame>
where
    Frame: frame::Backend,
{
    pub fn new(frame: Frame, bounds: Rectangle) -> Self {
        Self {
            frame,
            clip_stack: vec![],
            bounds,
            transform: geom::Transform::identity(),
        }
    }

    pub fn into_geometry(self) -> Frame::Geometry {
        self.frame.into_geometry()
    }

    fn transform_item(&self, item_transform: Option<&geom::Transform>) -> geom::Transform {
        match item_transform {
            Some(i) => i.post_concat(self.transform),
            None => self.transform,
        }
    }
}

impl<Frame> eidoplot::render::Surface for IcedSurface<Frame>
where
    Frame: frame::Backend,
{
    fn prepare(&mut self, size: geom::Size) -> Result<(), render::Error> {
        // scale up or down to fit the size into bounds, preserving aspect ratio and centering
        let tx = self.bounds.x;
        let ty = self.bounds.y;
        let sx = self.bounds.width / size.width();
        let sy = self.bounds.height / size.height();
        let s = sx.min(sy);
        let w = size.width() * s;
        let h = size.height() * s;
        let tx = tx + (self.bounds.width - w) / 2.0;
        let ty = ty + (self.bounds.height - h) / 2.0;
        self.transform =
            geom::Transform::from_translate(tx, ty).pre_concat(geom::Transform::from_scale(s, s));
        Ok(())
    }

    fn fill(&mut self, fill: render::Paint) -> Result<(), render::Error> {
        let color = match fill {
            render::Paint::Solid(c) => {
                iced::Color::from_rgba8(c.red(), c.green(), c.blue(), c.alpha() as f32 / 255.0)
            }
        };
        let bounds = self.bounds;
        self.frame
            .fill_rectangle(bounds.position(), bounds.size(), color);
        Ok(())
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), render::Error> {
        let transform = self.transform_item(path.transform);
        let iced_path = to_iced_path(&path.path, &transform);

        if let Some(fill) = path.fill.as_ref() {
            let iced_fill = to_iced_fill(fill);
            self.frame.fill(&iced_path, iced_fill);
        }

        if let Some(stroke) = path.stroke.as_ref() {
            let iced_stroke = to_iced_stroke(stroke);
            self.frame.stroke(&iced_path, iced_stroke);
        }

        Ok(())
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        let transform = self.transform_item(clip.transform);
        let iced_rect = to_iced_rect(&clip.rect, &transform);
        let mut clip = self.frame.draft(iced_rect);

        // make the clip frame the current frame
        std::mem::swap(&mut self.frame, &mut clip);

        self.clip_stack.push(clip);
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
        if let Some(mut clip) = self.clip_stack.pop() {
            // restore the previous frame
            std::mem::swap(&mut self.frame, &mut clip);
            // paste the clipped content back to the previous frame
            self.frame.paste(clip);
        }
        Ok(())
    }
}

#[inline]
fn to_iced_color(color: eidoplot::ColorU8) -> iced::Color {
    let [r, g, b, a] = color.rgba_f32();
    iced::Color::from_rgba(r, g, b, a)
}

#[inline]
fn to_iced_fill(paint: &render::Paint) -> geometry::Fill {
    match paint {
        render::Paint::Solid(color) => to_iced_color(*color).into(),
    }
}

#[inline]
fn to_iced_stroke<'a>(stroke: &'a render::Stroke) -> geometry::Stroke<'a> {
    let style = to_iced_color(stroke.color).into();
    let width = stroke.width;
    let line_dash = match &stroke.pattern {
        render::LinePattern::Solid => geometry::LineDash::default(),
        render::LinePattern::Dash(pattern) => geometry::LineDash {
            segments: *pattern,
            offset: 0,
        },
    };
    geometry::Stroke {
        width,
        style,
        line_dash,
        ..Default::default()
    }
}

#[inline]
fn to_iced_point(mut point: geom::Point, transform: &geom::Transform) -> iced::Point {
    transform.map_point(&mut point);
    iced::Point {
        x: point.x,
        y: point.y,
    }
}

fn to_iced_rect(rect: &geom::Rect, transform: &geom::Transform) -> iced::Rectangle {
    let mut tlbr = [
        geom::Point {
            x: rect.left(),
            y: rect.top(),
        },
        geom::Point {
            x: rect.right(),
            y: rect.bottom(),
        },
    ];
    transform.map_points(&mut tlbr);

    let [p1, p2] = tlbr;
    let x = p1.x.min(p2.x);
    let y = p1.y.min(p2.y);
    let width = (p2.x - p1.x).abs();
    let height = (p2.y - p1.y).abs();
    iced::Rectangle {
        x,
        y,
        width,
        height,
    }
}

fn to_iced_path(path: &geom::Path, transform: &geom::Transform) -> geometry::Path {
    geometry::Path::new(|builder| {
        let mut points = path.points().iter();
        for v in path.verbs() {
            match v {
                geom::PathVerb::Move => {
                    builder.move_to(to_iced_point(*points.next().unwrap(), transform));
                }
                geom::PathVerb::Line => {
                    builder.line_to(to_iced_point(*points.next().unwrap(), transform));
                }
                geom::PathVerb::Quad => {
                    let control = to_iced_point(*points.next().unwrap(), transform);
                    let to = to_iced_point(*points.next().unwrap(), transform);
                    builder.quadratic_curve_to(control, to);
                }
                geom::PathVerb::Cubic => {
                    let control_a = to_iced_point(*points.next().unwrap(), transform);
                    let control_b = to_iced_point(*points.next().unwrap(), transform);
                    let to = to_iced_point(*points.next().unwrap(), transform);
                    builder.bezier_curve_to(control_a, control_b, to);
                }
                geom::PathVerb::Close => {
                    builder.close();
                }
            }
        }
    })
}
