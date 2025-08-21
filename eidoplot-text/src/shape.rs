use ttf_parser as ttf;

use crate::Error;
use crate::font::{self, DatabaseExt, Font};

#[derive(Debug, Clone, Copy, Default)]
/// Text direction
pub enum Direction {
    #[default]
    LTR,
    RTL,
}

pub(crate) trait MainDirection {
    fn main_direction(&self) -> Direction;
}

#[cfg(debug_assertions)]
mod glyph_dbg {
    use std::fmt;

    const MAX_BYTES_PER_GLYPH: usize = 40;

    #[derive(Clone, Copy)]
    pub(crate) struct GlyphDbg {
        /// data storage for glyph character sequence
        chars_bytes: [u8; MAX_BYTES_PER_GLYPH],
        /// length of the character sequence
        chars_bytes_len: usize,
    }

    impl GlyphDbg {
        pub(super) fn new(chars: &str) -> Self {
            let mut chars_bytes_len = chars.len().min(MAX_BYTES_PER_GLYPH);
            while !chars.is_char_boundary(chars_bytes_len) {
                chars_bytes_len -= 1;
            }
            let mut chars_bytes = [0u8; MAX_BYTES_PER_GLYPH];
            chars_bytes[..chars_bytes_len].copy_from_slice(chars.as_bytes());
            Self {
                chars_bytes,
                chars_bytes_len,
            }
        }

        pub(crate) fn chars(&self) -> Option<&str> {
            // SAFETY: `chars_bytes` is valid UTF-8 as char boundaries are checked
            unsafe {
                Some(std::str::from_utf8_unchecked(
                    &self.chars_bytes[..self.chars_bytes_len],
                ))
            }
        }
    }

    impl fmt::Debug for GlyphDbg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let chars = self.chars().unwrap();
            write!(f, "GlyphDbg(\"{}\")", chars)
        }
    }
}

#[cfg(not(debug_assertions))]
mod glyph_dbg {
    #[derive(Debug, Clone, Copy)]
    pub(crate) struct GlyphDbg;
    impl GlyphDbg {
        pub(super) fn new(chars: &str) -> Self {
            GlyphDbg
        }
        pub(super) fn chars(&self) -> Option<&str> {
            None
        }
    }
}

pub(crate) use glyph_dbg::GlyphDbg;

/// Glyph info for a text shape
#[derive(Debug, Clone, Copy)]
pub(crate) struct Glyph {
    /// The id of the glyph in the font face
    pub(crate) id: ttf::GlyphId,
    /// The x-offset of the glyph in font units
    pub(crate) x_offset: i32,
    /// The y-offset of the glyph in font units
    pub(crate) y_offset: i32,
    /// The x-advance of the glyph in font units
    pub(crate) x_advance: i32,
    /// The y-advance of the glyph in font units
    pub(crate) y_advance: i32,
    /// The character of the glyph
    /// If the glyph is missing, this will be `None`
    pub(crate) dbg: GlyphDbg,
}

pub(crate) mod single_font {
    use super::{Direction, Glyph};
    use crate::font;

    /// Line info for a text shape
    #[derive(Debug, Clone)]
    pub(crate) struct Line {
        pub(crate) glyphs: Vec<Glyph>,
        /// The main direction of the line
        /// Most often a line has one unique direction.
        /// But if a line is bidirectional, this field is set to the first direction
        /// encountered, which will influence how the line is layed out.
        pub(super) main_dir: Direction,
    }

    impl super::MainDirection for Line {
        fn main_direction(&self) -> super::Direction {
            self.main_dir
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct Lines {
        pub(crate) lines: Vec<Line>,
        pub(crate) font: font::ID,
        pub(crate) metrics: font::FaceMetrics,
    }
}

pub(crate) mod fallback {
    use super::Direction;
    use crate::font;

    #[derive(Debug, Clone)]
    pub(crate) enum Glyph {
        Missing(String),
        Resolved(super::Glyph, font::ID, font::FaceMetrics),
    }

    impl Glyph {
        pub(super) fn missing_glyph(&self) -> Option<&str> {
            match self {
                Glyph::Missing(c) => Some(c.as_str()),
                Glyph::Resolved(..) => None,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct Line {
        pub(crate) glyphs: Vec<Glyph>,
        pub(super) main_dir: Direction,
    }

    impl Line {
        pub(super) fn fill_missing_chars(&mut self, other: &Line) {
            for (g1, g2) in self.glyphs.iter_mut().zip(other.glyphs.iter()) {
                let replace = match (&g1, &g2) {
                    (Glyph::Missing(_), Glyph::Resolved(..)) => true,
                    _ => false,
                };
                if replace {
                    *g1 = g2.clone();
                }
            }
        }
    }

    impl super::MainDirection for Line {
        fn main_direction(&self) -> super::Direction {
            self.main_dir
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct Lines {
        pub(crate) lines: Vec<Line>,
    }

    impl Lines {
        pub(super) fn replace_missing_chars(&mut self, other: &Lines) {
            for (l1, l2) in self.lines.iter_mut().zip(other.lines.iter()) {
                l1.fill_missing_chars(l2);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Lines {
    /// There is a single font used for all lines
    /// This is the most common case and allows optimizations
    SingleFont(single_font::Lines),
    /// The text could not be handled in a single font, and each glyph
    /// can have its own font
    Fallback(fallback::Lines),
}

impl Lines {
    fn have_fallback(&self) -> bool {
        matches!(self, Lines::Fallback { .. })
    }

    fn first_missing_glyph(&self) -> Option<&str> {
        match self {
            Lines::SingleFont(..) => None,
            Lines::Fallback(text) => text
                .lines
                .iter()
                .find_map(|l| l.glyphs.iter().find_map(|g| g.missing_glyph())),
        }
    }

    /// Checks whether `self` and `other` have the same shape,
    /// i.e. same number of lines and same number of glyphs per line
    fn same_shape(&self, other: &Lines) -> bool {
        match (self, other) {
            (Lines::SingleFont(ls1), Lines::SingleFont(ls2)) => {
                ls1.lines.len() == ls2.lines.len()
                    && ls1
                        .lines
                        .iter()
                        .zip(ls2.lines.iter())
                        .all(|(l1, l2)| l1.glyphs.len() == l2.glyphs.len())
            }
            (Lines::Fallback(ls1), Lines::Fallback(ls2)) => {
                ls1.lines.len() == ls2.lines.len()
                    && ls1
                        .lines
                        .iter()
                        .zip(ls2.lines.iter())
                        .all(|(l1, l2)| l1.glyphs.len() == l2.glyphs.len())
            }
            (Lines::SingleFont(ls1), Lines::Fallback(ls2)) => {
                ls1.lines.len() == ls2.lines.len()
                    && ls1
                        .lines
                        .iter()
                        .zip(ls2.lines.iter())
                        .all(|(l1, l2)| l1.glyphs.len() == l2.glyphs.len())
            }
            (Lines::Fallback(ls1), Lines::SingleFont(ls2)) => {
                ls1.lines.len() == ls2.lines.len()
                    && ls1
                        .lines
                        .iter()
                        .zip(ls2.lines.iter())
                        .all(|(l1, l2)| l1.glyphs.len() == l2.glyphs.len())
            }
        }
    }

    fn replace_missing_chars(&mut self, other: &Lines) {
        debug_assert!(self.same_shape(other));

        match (self, other) {
            (Lines::Fallback(t1), Lines::Fallback(t2)) => {
                t1.replace_missing_chars(t2);
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub(crate) lines: Lines,
    text: String,
    font: Font,
}

impl TextShape {
    pub fn shape_str(
        text: &str,
        font: &font::Font,
        db: &font::Database,
    ) -> Result<TextShape, Error> {
        let base_face_id = db
            .select_face_for_str(font, text)
            .or_else(|| db.select_face(font))
            .ok_or(Error::NoSuchFont(font.clone()))?;

        let mut shape = shape_text_with_font(text, base_face_id, font, db)?;

        if !shape.lines.have_fallback() {
            return Ok(shape);
        }

        let mut already_tried = Vec::new();

        loop {
            let Some(fallback_face_id) =
                db.select_face_fallback(shape.lines.first_missing_glyph().unwrap(), &already_tried)
            else {
                break;
            };

            let fallback_shape = shape_text_with_font(text, fallback_face_id, font, db)?;
            if fallback_shape.lines.first_missing_glyph().is_none() {
                // we replace the shape entirely with the fallback
                shape = fallback_shape;
                break;
            } else if fallback_shape.lines.same_shape(&shape.lines) {
                shape.lines.replace_missing_chars(&fallback_shape.lines);
                if !shape.lines.have_fallback() {
                    break;
                }
            }
            already_tried.push(fallback_face_id);
        }

        Ok(shape)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn font(&self) -> &Font {
        &self.font
    }
}

fn shape_text_with_font(
    text: &str,
    font_id: font::ID,
    font: &Font,
    db: &font::Database,
) -> Result<TextShape, Error> {
    db.with_face_data(font_id, |data, index| -> Result<TextShape, Error> {
        let mut face = ttf::Face::parse(data, index)?;
        font::apply_variations(&mut face, font);

        let metrics = font::face_metrics(&face);

        let hbface = rustybuzz::Face::from_face(face);
        let mut direction = None;
        let mut lines = Vec::new();
        let mut missing_glyphs = Vec::new();
        let mut buffer = rustybuzz::UnicodeBuffer::new();

        for line in text.lines() {
            let (line, buf) =
                shape_lines_with_font(line, &hbface, direction, &mut missing_glyphs, buffer)?;
            buffer = buf;

            direction = Some(line.main_dir);
            lines.push(line);
        }

        let lines = if missing_glyphs.is_empty() {
            Lines::SingleFont(single_font::Lines {
                lines: lines,
                font: font_id,
                metrics,
            })
        } else {
            // We have to switch to fallback mode.
            // There is exactly one glyph with id 0 per missing_glyphs entry.
            // Here we only switch the data structure, the fallback is implemented on
            // higher level.
            let mut fallback_lines = Vec::new();

            for l in lines {
                let mut fallback_glyphs = Vec::new();
                for g in l.glyphs {
                    if g.id.0 == 0 {
                        fallback_glyphs
                            .push(fallback::Glyph::Missing(missing_glyphs.pop().unwrap()));
                    } else {
                        fallback_glyphs.push(fallback::Glyph::Resolved(g, font_id, metrics));
                    }
                }
                fallback_lines.push(fallback::Line {
                    glyphs: fallback_glyphs,
                    main_dir: l.main_dir,
                });
            }

            Lines::Fallback(fallback::Lines {
                lines: fallback_lines,
            })
        };

        Ok(TextShape {
            lines,
            text: text.to_string(),
            font: font.clone(),
        })
    })
    .expect("Should be able to load that font")
}

fn empty_line(previous_dir: Option<Direction>) -> single_font::Line {
    // empty lines defaults to the previous line direction
    // or LTR if the first line is empty
    let main_dir = previous_dir.unwrap_or_default();
    single_font::Line {
        glyphs: Vec::new(),
        main_dir,
    }
}

// passing the rustybuzz buffer around is a bit hacky but allows us to reuse it
fn shape_lines_with_font(
    text: &str,
    hbface: &rustybuzz::Face,
    previous_dir: Option<Direction>,
    missing_glyphs: &mut Vec<String>,
    mut buffer: rustybuzz::UnicodeBuffer,
) -> Result<(single_font::Line, rustybuzz::UnicodeBuffer), Error> {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    assert!(
        bidi.paragraphs.len() <= 1,
        "Multiple paragraphs on the same line!"
    );

    let mut main_dir = None;

    let Some(para) = bidi.paragraphs.first() else {
        return Ok((empty_line(previous_dir), buffer));
    };

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
        if main_dir.is_none() {
            main_dir = Some(if ltr { Direction::LTR } else { Direction::RTL });
        }

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

            // FIXME: clusters should never be broken.
            // If a glyph is missing within a cluster, the
            // whole cluster should be excluded.
            // This case should be rare though, it can reasonably be assumed
            // that a face provides all the glyphs of a cluster.
            if info.glyph_id == 0 {
                missing_glyphs.push(sub_text[start..end].to_string());
            }
            let dbg = GlyphDbg::new(&sub_text[start..end]);

            assert!(info.glyph_id < 0x10000);

            glyphs.push(Glyph {
                id: ttf_parser::GlyphId(info.glyph_id as u16),
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
                dbg,
            });
        }

        buffer = output.clear();
    }

    // empty lines defaults to the previous line direction
    // or LTR if the first line is empty
    let main_dir = main_dir.or(previous_dir).unwrap_or_default();
    Ok((single_font::Line { glyphs, main_dir }, buffer))
}
