use ttf_parser as ttf;

use crate::Error;
use crate::font;
use crate::style;

#[derive(Debug, Clone, Copy)]
enum ResolvedGlyphId {
    Missing(char),
    Resolved(ttf::GlyphId),
}

/// A glyph positioned in a text shape
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    resolved_id: ResolvedGlyphId,
    x_offset: i32,
    y_offset: i32,
    x_advance: i32,
    y_advance: i32,
    font_id: font::ID,
    metrics: font::FaceMetrics,
}
impl Glyph {
    pub fn id(&self) -> Option<ttf::GlyphId> {
        match self.resolved_id {
            ResolvedGlyphId::Missing(_) => None,
            ResolvedGlyphId::Resolved(id) => Some(id),
        }
    }

    pub fn font_id(&self) -> font::ID {
        self.font_id
    }

    pub fn scale(&self, font_size: f32) -> f32 {
        self.metrics.scale(font_size)
    }

    pub fn x_offset(&self, font_size: f32) -> f32 {
        self.x_offset as f32 * self.metrics.scale(font_size)
    }

    pub fn y_offset(&self, font_size: f32) -> f32 {
        self.y_offset as f32 * self.metrics.scale(font_size)
    }

    pub fn x_advance(&self, font_size: f32) -> f32 {
        self.x_advance as f32 * self.metrics.scale(font_size)
    }

    pub fn has_x_advance(&self) -> bool {
        self.x_advance != 0
    }

    pub fn y_advance(&self, font_size: f32) -> f32 {
        self.y_advance as f32 * self.metrics.scale(font_size)
    }

    pub fn height(&self, font_size: f32) -> f32 {
        self.metrics.height(font_size)
    }
    
    pub fn x_height(&self, font_size: f32) -> f32 {
        self.metrics.x_height(font_size)
    }
    
    pub fn cap_height(&self, font_size: f32) -> f32 {
        self.metrics.cap_height(font_size)
    }
    
    pub fn line_gap(&self, font_size: f32) -> f32 {
        self.metrics.line_gap(font_size)
    }
}

impl Glyph {
    fn missing_char(&self) -> Option<char> {
        match self.resolved_id {
            ResolvedGlyphId::Missing(c) => Some(c),
            ResolvedGlyphId::Resolved(_) => None,
        }
    }
}

/// A single line of text, layed-out in font units
#[derive(Debug, Clone)]
pub struct Line {
    glyphs: Vec<Glyph>,
    style: style::Font,
}

impl Line {
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    pub fn style(&self) -> &style::Font {
        &self.style
    }

    pub fn width(&self, font_size: f32) -> f32 {
        self.glyphs.iter().map(|g| g.x_advance(font_size)).sum()
    }

    pub fn height(&self, font_size: f32) -> f32 {
        self.glyphs
            .iter()
            .map(|g| g.height(font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn x_height(&self, font_size: f32) -> f32 {
        self.glyphs
            .iter()
            .map(|g| g.x_height(font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn cap_height(&self, font_size: f32) -> f32 {
        self.glyphs
            .iter()
            .map(|g| g.x_height(font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn gap(&self, font_size: f32) -> f32 {
        self.glyphs
            .iter()
            .map(|g| g.line_gap(font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

/// A text shape that is layed-out in font units
#[derive(Debug, Clone)]
pub struct Text {
    lines: Vec<Line>,
    text: String,
    style: style::Font,
}

impl Text {
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn width(&self, font_size: f32) -> f32 {
        self.lines
            .iter()
            .map(|l| l.width(font_size))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    pub fn height(&self, font_size: f32) -> f32 {
        let mut h = 0.0;
        for l in self.lines.iter().take(self.lines.len() - 1) {
            h += l.height(font_size) + l.gap(font_size);
        }
        if let Some(l) = self.lines.last() {
            h += l.height(font_size);
        }
        h
    }
}

impl Text {
    fn first_missing_char(&self) -> Option<char> {
        self.lines
            .iter()
            .find_map(|l| l.glyphs.iter().find_map(|g| g.missing_char()))
    }

    fn missing_chars(&self) -> impl Iterator<Item = char> {
        self.lines
            .iter()
            .flat_map(|l| l.glyphs.iter().filter_map(|g| g.missing_char()))
    }

    fn same_shape(&self, other: &Text) -> bool {
        self.lines.len() == other.lines.len()
            && self
                .lines
                .iter()
                .zip(other.lines.iter())
                .all(|(a, b)| a.glyphs.len() == b.glyphs.len())
    }

    fn fill_missing_chars(&mut self, other: &Text) {
        for (l, ol) in self.lines.iter_mut().zip(other.lines.iter()) {
            for (g, og) in l.glyphs.iter_mut().zip(ol.glyphs.iter()) {
                let replace = match (g.resolved_id, og.resolved_id) {
                    (ResolvedGlyphId::Missing(_), ResolvedGlyphId::Resolved(_)) => true,
                    _ => false,
                };
                if replace {
                    *g = *og;
                }
            }
        }
    }
}

pub fn shape_text(
    text: &str,
    style: &style::Font,
    db: &font::Database,
) -> Result<Text, Error> {
    let base_face_id = font::select_face(db, style).ok_or(Error::NoSuchFont(style.clone()))?;
    let mut shape = shape_text_with_font(text, base_face_id, style, db)?;

    let mut missing = shape.first_missing_char();
    if missing.is_none() {
        return Ok(shape);
    }

    let mut already_tried = vec![base_face_id];

    while let Some(c) = missing {
        let Some(fallback_id) = font::select_face_fallback(db, c, &already_tried) else {
            break;
        };

        let fallback_shape = shape_text_with_font(text, fallback_id, style, db)?;
        if fallback_shape.first_missing_char().is_none() {
            // we replace the shape entirely with the fallback
            shape = fallback_shape;
            break;
        } else if fallback_shape.same_shape(&shape) {
            // we replace the shape with the fallback
            shape.fill_missing_chars(&fallback_shape);
            missing = shape.first_missing_char();
        }
        already_tried.push(fallback_id);
    }

    for (i, c) in shape.missing_chars().enumerate() {
        if i == 0 {
            println!("text {:?} has missing chars:", text);
        }
        println!("  missing char: '{}' ({})", c, c as u32);
    }

    Ok(shape)
}

fn shape_text_with_font(
    text: &str,
    font_id: font::ID,
    style: &style::Font,
    db: &font::Database,
) -> Result<Text, Error> {
    db.with_face_data(font_id, |data, index| -> Result<Text, Error> {
        let mut face = ttf::Face::parse(data, index)?;
        font::apply_variations(&mut face, style);

        let metrics = font::face_metrics(&face);

        let hbface = rustybuzz::Face::from_face(face);
        let mut buffer = rustybuzz::UnicodeBuffer::new();

        let mut lines = Vec::new();

        for line in text.lines() {
            buffer = shape_lines_with_font(line, font_id, &hbface, metrics, buffer, style, &mut lines)?;
        }

        Ok(Text {
            lines,
            text: text.to_string(),
            style: style.clone(),
        })
    })
    .expect("Should be able to load that font")
}

// passing the rustybuzz buffer around is a bit hacky but allows us to reuse it
fn shape_lines_with_font(
    text: &str,
    font_id: font::ID,
    hbface: &rustybuzz::Face,
    metrics: font::FaceMetrics,
    mut buffer: rustybuzz::UnicodeBuffer,
    style: &style::Font,
    lines: &mut Vec<Line>,
) -> Result<rustybuzz::UnicodeBuffer, Error> {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    if bidi.paragraphs.len() > 1 {
        println!("Multiple paragraphs on the same line. Issueing multiple lines!");
    }
    for para in bidi.paragraphs.iter() {
        let line = para.range.clone();
        let (levels, runs) = bidi.visual_runs(para, line.clone());

        let mut glyphs = Vec::new();

        for run in runs.iter() {
            let sub_text = &text[run.clone()];
            if sub_text.is_empty() {
                continue;
            }

            let ltr = levels[run.start].is_ltr();
            let hb_direction = if ltr {
                rustybuzz::Direction::LeftToRight
            } else {
                rustybuzz::Direction::RightToLeft
            };

            buffer.push_str(sub_text);
            buffer.set_direction(hb_direction);

            let features = &[rustybuzz::Feature::new(
                ttf::Tag::from_bytes(b"kern"),
                1,
                ..,
            )];

            let output = rustybuzz::shape(&hbface, features, buffer);

            let positions = output.glyph_positions();
            let infos = output.glyph_infos();

            for i in 0..output.len() {
                let pos = positions[i];
                let info = infos[i];

                let start = info.cluster as usize;
                let end = if ltr {
                    i.checked_add(1)
                } else {
                    i.checked_sub(1)
                }
                .and_then(|last| infos.get(last))
                .map_or(sub_text.len(), |info| info.cluster as usize);

                let resolved_id = if info.glyph_id == 0 {
                    ResolvedGlyphId::Missing(sub_text[start..end].chars().next().unwrap())
                } else {
                    ResolvedGlyphId::Resolved(ttf::GlyphId(info.glyph_id as u16))
                };

                glyphs.push(Glyph {
                    resolved_id,
                    x_offset: pos.x_offset,
                    y_offset: pos.y_offset,
                    x_advance: pos.x_advance,
                    y_advance: pos.y_advance,
                    font_id,
                    metrics,
                });
            }

            buffer = output.clear();
        }

        lines.push(Line {
            glyphs,
            style: style.clone(),
            // range: line.clone(),
        });
    }
    Ok(buffer)
}
