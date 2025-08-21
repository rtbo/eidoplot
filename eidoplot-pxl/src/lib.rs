use std::{io, sync::Arc};

use tiny_skia::{self, FillRule, Mask, Pixmap, PixmapMut};

use eidoplot::{geom, render, style};

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

    pub fn save_png(&self, path: &str) -> io::Result<()> {
        self.pixmap.save_png(path)?;
        Ok(())
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

    fn fill(&mut self, px: &mut PixmapMut<'_>, fill: style::Fill) -> Result<(), render::Error> {
        match fill {
            style::Fill::Solid(color) => {
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
        // FIXME: error management
        let layout = eidoplot_text::shape_and_layout_str(
            text.text,
            text.font,
            &self.fontdb,
            text.font_size,
            &text.options,
        )
        .unwrap();

        let mut paint = tiny_skia::Paint::default();
        ts_fill(text.fill, &mut paint);
        let render_opts = eidoplot_text::render::Options {
            fill: Some(paint),
            outline: None,
            transform: ts_text,
            mask: self.clip.as_ref(),
        };
        let db = &self.fontdb;
        eidoplot_text::render::render_text_tiny_skia(&layout, &render_opts, db, px);

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
        ts_fill(text.fill, &mut paint);
        let render_opts = eidoplot_text::render::Options {
            fill: Some(paint),
            outline: None,
            transform: ts_text,
            mask: self.clip.as_ref(),
        };
        let db = &self.fontdb;
        eidoplot_text::render::render_text_tiny_skia(&text.layout, &render_opts, db, px);

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

    fn fill(&mut self, fill: style::Fill) -> Result<(), render::Error> {
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

    fn fill(&mut self, fill: style::Fill) -> Result<(), render::Error> {
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

fn ts_color(color: style::Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(color.red(), color.green(), color.blue(), color.alpha())
}

fn ts_fill(fill: style::Fill, paint: &mut tiny_skia::Paint) {
    paint.colorspace = tiny_skia::ColorSpace::Gamma2;
    match fill {
        style::Fill::Solid(color) => {
            let color = ts_color(color);
            paint.set_color(color);
        }
    }
}

fn ts_stroke(stroke: style::Line, paint: &mut tiny_skia::Paint) -> tiny_skia::Stroke {
    let color = ts_color(stroke.color);
    paint.colorspace = tiny_skia::ColorSpace::Gamma2;
    paint.set_color(color);

    let mut ts = tiny_skia::Stroke {
        width: stroke.width,
        ..Default::default()
    };

    match stroke.pattern {
        style::LinePattern::Solid => (),
        style::LinePattern::Dash(dash) => {
            ts.dash = tiny_skia::StrokeDash::new(vec![dash.0, dash.1], 0.0);
        }
        style::LinePattern::Dot => {
            ts.dash = tiny_skia::StrokeDash::new(vec![1.0, 1.0], 0.0);
        }
    }
    ts
}
