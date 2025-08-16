use core::ops::Range;

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
struct Glyph {
    resolved_id: ResolvedGlyphId,
    x_offset: i32,
    y_offset: i32,
    x_advance: i32,
    y_advance: i32,
    font_id: font::ID,
    metrics: font::FaceMetrics,
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
struct LineShape {
    glyphs: Vec<Glyph>,
    range: Range<usize>,
}

/// A text shape that is layed-out in font units
#[derive(Debug, Clone)]
pub struct TextShape {
    lines: Vec<LineShape>,
    text: String,
    style: style::Font,
}

impl TextShape {
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

    fn same_shape(&self, other: &TextShape) -> bool {
        self.lines.len() == other.lines.len()
            && self
                .lines
                .iter()
                .zip(other.lines.iter())
                .all(|(a, b)| a.glyphs.len() == b.glyphs.len())
    }

    fn fill_missing_chars(&mut self, other: &TextShape) {
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
) -> Result<TextShape, Error> {
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
) -> Result<TextShape, Error> {
    db.with_face_data(font_id, |data, index| -> Result<TextShape, Error> {
        let mut face = ttf::Face::parse(data, index)?;
        font::apply_variations(&mut face, style);

        let metrics = font::face_metrics(&face);

        let hbface = rustybuzz::Face::from_face(face);
        let mut buffer = rustybuzz::UnicodeBuffer::new();

        let mut lines = Vec::new();

        for line in text.lines() {
            buffer = shape_lines_with_font(line, font_id, &hbface, metrics, buffer, &mut lines)?;
        }

        Ok(TextShape {
            lines,
            text: text.to_string(),
            style: style.clone(),
        })
    })
    .expect("Should be able to load that font")
}

// passing the rustybuzz buffer is a bit hacky but allows us to reuse it
fn shape_lines_with_font(
    text: &str,
    font_id: font::ID,
    hbface: &rustybuzz::Face,
    metrics: font::FaceMetrics,
    mut buffer: rustybuzz::UnicodeBuffer,
    lines: &mut Vec<LineShape>,
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

        lines.push(LineShape {
            glyphs,
            range: line.clone(),
        });
    }
    Ok(buffer)
}
