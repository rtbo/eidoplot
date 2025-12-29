use std::io;

use eidoplot::{ColorU8, geom, render};
use tiny_skia::{self, FillRule, Mask, Pixmap, PixmapMut};

#[derive(Debug, Clone)]
pub struct PxlSurface {
    pixmap: Pixmap,
    state: State,
}

impl PxlSurface {
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let pixmap = Pixmap::new(width, height)?;
        let state = State::new(width, height);
        Some(Self { pixmap, state })
    }

    pub fn save_png<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        self.pixmap.save_png(path)?;
        Ok(())
    }

    pub fn into_pixmap(self) -> Pixmap {
        self.pixmap
    }
}

pub struct PxlSurfaceRef<'a> {
    pixmap: PixmapMut<'a>,
    state: State,
}

impl<'a> PxlSurfaceRef<'a> {
    pub fn from_pixmap_mut(pixmap: PixmapMut<'a>) -> Self {
        let state = State::new(pixmap.width(), pixmap.height());
        Self { pixmap, state }
    }

    pub fn from_bytes(bytes: &'a mut [u8], width: u32, height: u32) -> Option<Self> {
        let pixmap = PixmapMut::from_bytes(bytes, width, height)?;
        let state = State::new(pixmap.width(), pixmap.height());
        Some(Self { pixmap, state })
    }

    pub fn save_png(&self, path: &str) -> io::Result<()> {
        self.pixmap.as_ref().save_png(path)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct State {
    width: u32,
    height: u32,
    transform: geom::Transform,
    clip: Option<Mask>,
}

impl State {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            transform: geom::Transform::identity(),
            clip: None,
        }
    }

    fn prepare(&mut self, size: geom::Size) {
        let sx = self.width as f32 / size.width();
        let sy = self.height as f32 / size.height();
        self.transform = geom::Transform::from_scale(sx, sy);
    }

    fn fill(&mut self, px: &mut PixmapMut<'_>, fill: render::Paint) {
        match fill {
            render::Paint::Solid(color) => {
                let color = ts_color(color);
                px.fill(color);
            }
        }
    }

    fn draw_path(
        &mut self,
        px: &mut PixmapMut<'_>,
        path: &render::Path,
    )  {
        let transform = path
            .transform
            .map(|t| t.post_concat(self.transform))
            .unwrap_or(self.transform);

        if let Some(fill) = path.fill {
            let mut paint = tiny_skia::Paint::default();
            ts_fill(fill, &mut paint);

            px.fill_path(
                path.path,
                &paint,
                tiny_skia::FillRule::Winding,
                transform,
                self.clip.as_ref(),
            );
        }
        if let Some(stroke) = path.stroke {
            let mut paint = tiny_skia::Paint::default();
            let stroke = ts_stroke(stroke, &mut paint);
            px.stroke_path(path.path, &paint, &stroke, transform, self.clip.as_ref());
        }
    }

    fn push_clip(&mut self, clip: &render::Clip)  {
        if self.clip.is_some() {
            unimplemented!("clip with more than 1 layer");
        } else {
            let mut mask = Mask::new(self.width, self.height).unwrap();
            let transform = clip
                .transform
                .map(|t| t.post_concat(self.transform))
                .unwrap_or(self.transform);
            let path = clip.rect.to_path();
            mask.fill_path(&path, FillRule::Winding, true, transform);
            self.clip = Some(mask);
        }
    }

    fn pop_clip(&mut self)  {
        self.clip = None;
    }
}

impl render::Surface for PxlSurface {
    fn prepare(&mut self, size: geom::Size)  {
        self.state.prepare(size)
    }

    fn fill(&mut self, fill: render::Paint) {
        let mut px = self.pixmap.as_mut();
        self.state.fill(&mut px, fill)
    }

    fn draw_path(&mut self, path: &render::Path) {
        let mut px = self.pixmap.as_mut();
        self.state.draw_path(&mut px, path)
    }

    fn push_clip(&mut self, clip: &render::Clip) {
        self.state.push_clip(clip)
    }

    fn pop_clip(&mut self)  {
        self.state.pop_clip()
    }
}

impl render::Surface for PxlSurfaceRef<'_> {
    fn prepare(&mut self, size: geom::Size) {
        self.state.prepare(size)
    }

    fn fill(&mut self, fill: render::Paint) {
        self.state.fill(&mut self.pixmap, fill)
    }

    fn draw_path(&mut self, path: &render::Path) {
        self.state.draw_path(&mut self.pixmap, path)
    }

    fn push_clip(&mut self, clip: &render::Clip) {
        self.state.push_clip(clip)
    }

    fn pop_clip(&mut self) {
        self.state.pop_clip()
    }
}

fn ts_color(color: ColorU8) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(color.red(), color.green(), color.blue(), color.alpha())
}

fn ts_fill(fill: render::Paint, paint: &mut tiny_skia::Paint) {
    match fill {
        render::Paint::Solid(color) => {
            let color = ts_color(color);
            paint.set_color(color);
        }
    }
    paint.force_hq_pipeline = true;
}

fn ts_stroke(stroke: render::Stroke, paint: &mut tiny_skia::Paint) -> tiny_skia::Stroke {
    paint.force_hq_pipeline = true;

    let color = ts_color(stroke.color);
    paint.set_color(color);

    let mut ts = tiny_skia::Stroke {
        width: stroke.width,
        ..Default::default()
    };

    match stroke.pattern {
        render::LinePattern::Solid => (),
        render::LinePattern::Dash(dash) => {
            let array = dash.iter().map(|d| d * stroke.width).collect();
            ts.dash = tiny_skia::StrokeDash::new(array, 0.0);
        }
    }
    ts
}
