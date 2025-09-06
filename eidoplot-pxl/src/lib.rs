use std::io;
use std::sync::Arc;

use eidoplot::{geom, render, style};
use eidoplot_text as text;
use text::fontdb;
use tiny_skia::{self, FillRule, Mask, Pixmap, PixmapMut};

const DEBUG_TEXT_BBOX: bool = false;

#[derive(Debug, Clone)]
pub struct PxlSurface {
    pixmap: Pixmap,
    state: State,
}

impl PxlSurface {
    pub fn new(width: u32, height: u32, fontdb: Option<Arc<fontdb::Database>>) -> Option<Self> {
        let pixmap = Pixmap::new(width, height)?;
        let state = State::new(width, height, fontdb);
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
    pub fn from_pixmap_mut(pixmap: PixmapMut<'a>, fontdb: Option<Arc<fontdb::Database>>) -> Self {
        let state = State::new(pixmap.width(), pixmap.height(), fontdb);
        Self { pixmap, state }
    }

    pub fn from_bytes(
        bytes: &'a mut [u8],
        width: u32,
        height: u32,
        fontdb: Option<Arc<fontdb::Database>>,
    ) -> Option<Self> {
        let pixmap = PixmapMut::from_bytes(bytes, width, height)?;
        let state = State::new(pixmap.width(), pixmap.height(), fontdb);
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
    fontdb: Arc<fontdb::Database>,
    transform: geom::Transform,
    clip: Option<Mask>,
}

impl State {
    fn new(width: u32, height: u32, fontdb: Option<Arc<fontdb::Database>>) -> Self {
        let fontdb = fontdb.unwrap_or_else(|| Arc::new(eidoplot::bundled_font_db()));
        Self {
            width,
            height,
            fontdb,
            transform: geom::Transform::identity(),
            clip: None,
        }
    }

    fn prepare(&mut self, size: geom::Size) -> Result<(), render::Error> {
        let sx = self.width as f32 / size.width();
        let sy = self.height as f32 / size.height();
        self.transform = geom::Transform::from_scale(sx, sy);
        Ok(())
    }

    fn fill(&mut self, px: &mut PixmapMut<'_>, fill: render::Paint) -> Result<(), render::Error> {
        match fill {
            render::Paint::Solid(color) => {
                let color = ts_color(color);
                px.fill(color);
            }
        }
        Ok(())
    }

    fn draw_rect(
        &mut self,
        px: &mut PixmapMut<'_>,
        rect: &render::Rect,
    ) -> Result<(), render::Error> {
        let path = rect.rect.to_path();
        let path = render::Path {
            path: &path,
            fill: rect.fill,
            stroke: rect.stroke,
            transform: rect.transform,
        };
        self.draw_path(px, &path)?;
        Ok(())
    }

    fn draw_path(
        &mut self,
        px: &mut PixmapMut<'_>,
        path: &render::Path,
    ) -> Result<(), render::Error> {
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
        Ok(())
    }

    fn draw_text(
        &mut self,
        px: &mut PixmapMut<'_>,
        text: &render::Text,
    ) -> Result<(), render::Error> {
        let ts_text = text
            .transform
            .map(|t| t.post_concat(self.transform))
            .unwrap_or(self.transform);
        let layout = text::shape_and_layout_str(
            text.text,
            text.font,
            &self.fontdb,
            text.font_size,
            &text.options,
        )?;

        let mut paint = tiny_skia::Paint::default();
        ts_text_fill(text.fill, &mut paint);
        let render_opts = text::render::Options {
            fill: Some(paint),
            outline: None,
            transform: ts_text,
            mask: self.clip.as_ref(),
        };
        let db = &self.fontdb;
        text::render::render_text_tiny_skia(&layout, &render_opts, db, px);

        if DEBUG_TEXT_BBOX {
            self.draw_text_bbox(px, layout.bbox(), ts_text)?;
        }

        Ok(())
    }

    fn draw_text_layout(
        &mut self,
        px: &mut PixmapMut<'_>,
        text: &render::TextLayout,
    ) -> Result<(), render::Error> {
        let ts_text = text
            .transform
            .map(|t| t.post_concat(self.transform))
            .unwrap_or(self.transform);

        let mut paint = tiny_skia::Paint::default();
        ts_text_fill(text.fill, &mut paint);
        let render_opts = text::render::Options {
            fill: Some(paint),
            outline: None,
            transform: ts_text,
            mask: self.clip.as_ref(),
        };
        let db = &self.fontdb;
        text::render_text_tiny_skia(&text.layout, &render_opts, db, px);

        if DEBUG_TEXT_BBOX {
            self.draw_text_bbox(px, text.layout.bbox(), ts_text)?;
        }

        Ok(())
    }

    fn draw_text_bbox(
        &mut self,
        px: &mut PixmapMut<'_>,
        bbox: eidoplot_text::BBox,
        transform: geom::Transform,
    ) -> Result<(), render::Error> {
        let color = eidoplot::style::color::RED;
        let stroke = eidoplot::style::Line {
            width: 1.0,
            color,
            pattern: eidoplot::style::LinePattern::Solid,
            opacity: None,
        };
        let eidoplot_text::BBox {
            top,
            right,
            bottom,
            left,
        } = bbox;
        let rect = geom::Rect::from_trbl(top, right, bottom, left);
        let rrect = render::Rect {
            rect,
            fill: None,
            stroke: Some(stroke.as_stroke(&())),
            transform: Some(&transform),
        };
        self.draw_rect(px, &rrect)?;
        Ok(())
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        if self.clip.is_some() {
            unimplemented!("clip with more than 1 layer");
        } else {
            let mut mask = Mask::new(self.width, self.height).unwrap();
            let transform = clip
                .transform
                .map(|t| t.post_concat(self.transform))
                .unwrap_or(self.transform);
            mask.fill_path(&clip.path, FillRule::Winding, true, transform);
            self.clip = Some(mask);
        }
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
        self.clip = None;
        Ok(())
    }
}

impl render::Surface for PxlSurface {
    fn prepare(&mut self, size: geom::Size) -> Result<(), render::Error> {
        self.state.prepare(size)
    }

    fn fill(&mut self, fill: render::Paint) -> Result<(), render::Error> {
        let mut px = self.pixmap.as_mut();
        self.state.fill(&mut px, fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), render::Error> {
        let mut px = self.pixmap.as_mut();
        self.state.draw_rect(&mut px, rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), render::Error> {
        let mut px = self.pixmap.as_mut();
        self.state.draw_path(&mut px, path)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        let mut px = self.pixmap.as_mut();
        self.state.draw_text(&mut px, text)
    }

    fn draw_text_layout(&mut self, text: &render::TextLayout) -> Result<(), render::Error> {
        let mut px = self.pixmap.as_mut();
        self.state.draw_text_layout(&mut px, text)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        self.state.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
        self.state.pop_clip()
    }
}

impl render::Surface for PxlSurfaceRef<'_> {
    fn prepare(&mut self, size: geom::Size) -> Result<(), render::Error> {
        self.state.prepare(size)
    }

    fn fill(&mut self, fill: render::Paint) -> Result<(), render::Error> {
        self.state.fill(&mut self.pixmap, fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), render::Error> {
        self.state.draw_rect(&mut self.pixmap, rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), render::Error> {
        self.state.draw_path(&mut self.pixmap, path)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        self.state.draw_text(&mut self.pixmap, text)
    }

    fn draw_text_layout(&mut self, text: &render::TextLayout) -> Result<(), render::Error> {
        self.state.draw_text_layout(&mut self.pixmap, text)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        self.state.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
        self.state.pop_clip()
    }
}

fn ts_color(color: style::ColorU8) -> tiny_skia::Color {
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

fn ts_text_fill(fill: render::Paint, paint: &mut tiny_skia::Paint) {
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
