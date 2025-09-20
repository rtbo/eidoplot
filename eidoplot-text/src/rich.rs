use crate::font;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A set of properties to be applied to a text span
#[derive(Debug, Clone)]
pub struct TextOptProps {
    font: Option<font::Font>,
    font_size: Option<f32>,
    fill: Option<Color>,
    stroke: Option<Color>,
    underline: Option<bool>,
    strikeout: Option<bool>,
}

/// A set of resolved properties for a text span
#[derive(Debug, Clone)]
pub struct TextProps {
    font: font::Font,
    font_size: f32,
    fill: Option<Color>,
    stroke: Option<Color>,
    underline: bool,
    strikeout: bool,
}

impl TextProps {
    pub fn new(font_size: f32) -> TextProps {
        TextProps {
            font: font::Font::default(),
            font_size,
            fill: Some(Color { r: 0, g: 0, b: 0, a: 255 }),
            stroke: None,
            underline: false,
            strikeout: false,
        }
    }

    fn apply_opts(&mut self, opts: &TextOptProps) {
        if let Some(font) = &opts.font {
            self.font = font.clone();
        }
        if let Some(font_size) = opts.font_size {
            self.font_size = font_size;
        }
        if let Some(fill) = opts.fill {
            self.fill = Some(fill);
        }
        if let Some(stroke) = opts.stroke {
            self.stroke = Some(stroke);
        }
        if let Some(underline) = opts.underline {
            self.underline = underline;
        }
        if let Some(strikeout) = opts.strikeout {
            self.strikeout = strikeout;
        }
    }
}

/// A text span
#[derive(Debug, Clone)]
struct TextSpan {
    start: usize,
    end: usize,
    props: TextOptProps,
}

/// A builder struct for rich text
#[derive(Debug, Clone)]
pub struct RichTextBuilder {
    text: String,
    init_props: TextProps,
    spans: Vec<TextSpan>,
}

impl RichTextBuilder {
    /// Create a new RichTextBuilder
    pub fn new(text: String, init_props: TextProps) -> RichTextBuilder {
        RichTextBuilder {
            text,
            init_props,
            spans: vec![],
        }
    }

    /// Add a new text span
    pub fn add_span(&mut self, start: usize, end: usize, props: TextOptProps) {
        assert!(start <= end);
        assert!(end <= self.text.len());
        self.spans.push(TextSpan { start, end, props });
    }

    /// Create a RichTextLayout from this builder
    pub fn done(self) -> RichTextLayout {
        // Visual hints about what this function does.
        // "Some RICH text string"
        //       ^   ^              spans[0]: boldexport CMAKE_POLICY_VERSION_MINIMUM=3.5
        //         ^      ^         spans[1]: underline
        // "_____bbBBuuuuu_______"  (B = bold + underline)
        //                          ACTION              STORE
        //  _                       noop                [false, false]
        //   _                      noop                [false, false]
        //    _                     noop                [false, false]
        //     _                    noop                [false, false]
        //      _                   noop                [false, false]
        //       b                  apply bold          [true, false]
        //        b                 noop                [true, false]
        //         B                apply uline         [true, true]
        //          B               noop                [true, true]
        //           u              remove bold         [false, true]
        //            u             noop                [false, true]
        //             u            noop                [false, true]
        //              u           noop                [false, true]
        //               u          noop                [false, true]
        //                _         remove uline        [false, false]
        //                 _        noop                [false, false]
        //                  _       noop                [false, false]
        //                   _      noop                [false, false]
        //                    _     noop                [false, false]
        //                     _    noop                [false, false]
        //                      _   noop                [false, false]
        let RichTextBuilder {
            text,
            init_props,
            mut spans,
        } = self;

        spans.sort_unstable_by_key(|s| (s.start, s.end));

        let mut props_store = vec![false; spans.len()];
        let mut cur_span = ResolvedTextSpan {
            start: 0,
            end: 0,
            props: init_props.clone(),
        };
        let mut resolved_spans = Vec::new();

        for (ci, _) in text.char_indices() {
            for (si, span) in spans.iter().enumerate() {
                if span.start == ci {
                    if cur_span.start != cur_span.end {
                        resolved_spans.push(cur_span.clone());
                        cur_span.start = cur_span.end;
                    }
                    props_store[si] = true;
                    cur_span.props = init_props.clone();
                    for (i, p) in props_store.iter().enumerate() {
                        if *p {
                            cur_span.props.apply_opts(&spans[i].props);
                        }
                    }
                }
                if span.end == ci {
                    if cur_span.start != cur_span.end {
                        resolved_spans.push(cur_span.clone());
                        cur_span.start = cur_span.end;
                    }
                    props_store[si] = false;
                    cur_span.props = init_props.clone();
                    for (i, p) in props_store.iter().enumerate() {
                        if *p {
                            cur_span.props.apply_opts(&spans[i].props);
                        }
                    }
                }
            }
            cur_span.end = ci;
        }
        if cur_span.start != cur_span.end {
            resolved_spans.push(cur_span.clone());
        }

        RichTextLayout {
            text,
            spans: resolved_spans,
        }
    }
}

#[derive(Debug, Clone)]
struct ResolvedTextSpan {
    start: usize,
    end: usize,
    props: TextProps,
}

#[derive(Debug, Clone)]
pub struct RichTextLayout {
    text: String,
    spans: Vec<ResolvedTextSpan>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let mut builder = RichTextBuilder::new("Some RICH text string".to_string(), TextProps::new(12.0));
        //builder.add_span(0, 5, TextOptProps::new().bold());
    }
}
