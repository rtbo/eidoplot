/*!
 * # Declarative design module for plotive
 *
 * This module contains all data structures for the design of plotting figures.
 */
pub mod annot;
pub mod axis;
pub mod figure;
pub mod legend;
pub mod plot;
pub mod series;

pub use annot::Annotation;
pub use axis::Axis;
pub use figure::{FigLegend, Figure};
pub use legend::Legend;
pub use plot::{Plot, PlotLegend, Subplots};
pub use series::{DataCol, Series, data_inline, data_src_ref};

/// Index of a plot in a subplot grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlotIdx {
    /// Row index of the plot (0-based)
    pub row: u32,
    /// Column index of the plot (0-based)
    pub col: u32,
}

impl PlotIdx {
    /// Create a new PlotIdx from row and column indices
    pub fn new(row: u32, col: u32) -> Self {
        PlotIdx { row, col }
    }

    pub(crate) fn index(&self, cols: u32) -> usize {
        (self.row * cols + self.col) as usize
    }

    pub(crate) fn is_first(&self) -> bool {
        self.row == 0 && self.col == 0
    }

    pub(crate) fn next(&self, cols: u32) -> Self {
        let mut row = self.row;
        let mut col = self.col + 1;
        if col >= cols {
            col = 0;
            row += 1;
        }
        PlotIdx { row, col }
    }
}

/// Convert a (row, col) tuple into a PlotIdx
impl From<(u32, u32)> for PlotIdx {
    fn from((row, col): (u32, u32)) -> Self {
        PlotIdx { row, col }
    }
}

/// Iterator over all PlotIdx in a subplot grid
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotIdxIter {
    rows: u32,
    cols: u32,
    current: PlotIdx,
    done: bool,
}

impl PlotIdxIter {
    pub(crate) fn new(rows: u32, cols: u32) -> Self {
        PlotIdxIter {
            rows,
            cols,
            current: PlotIdx { row: 0, col: 0 },
            done: rows == 0 || cols == 0,
        }
    }
}

impl Iterator for PlotIdxIter {
    type Item = PlotIdx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let result = self.current;
        if self.current.col + 1 >= self.cols {
            self.current.col = 0;
            self.current.row += 1;
            if self.current.row >= self.rows {
                self.done = true;
            }
        } else {
            self.current.col += 1;
        }
        Some(result)
    }
}

impl std::iter::FusedIterator for PlotIdxIter {}

// Structs defined with this macro use theme::Color for the generic color of rich properties
// Caller must impl a specific Default for the $props_struct.
macro_rules! define_rich_text_structs {
    ($text_struct:ident, $props_struct:ident, $opt_props_struct:ident) => {
        /// Rich text properties that can apply only some properties on a given text span
        pub type $opt_props_struct = $crate::text::rich::TextOptProps<$crate::style::theme::Color>;

        /// Rich text base properties with plotive theme colors
        #[derive(Debug, Clone)]
        pub struct $props_struct($crate::text::rich::TextProps<$crate::style::theme::Color>);

        impl $props_struct {
            fn new(font_size: f32) -> Self {
                Self(
                    $crate::text::rich::TextProps::new(font_size)
                        .with_font($crate::style::defaults::FONT_FAMILY.parse().unwrap()),
                )
            }

            /// Set the font properties and return self for chaining
            pub fn with_font(self, font: $crate::text::font::Font) -> Self {
                Self(self.0.with_font(font))
            }

            /// Set the text fill color and return self for chaining
            pub fn with_fill(self, fill: Option<$crate::style::theme::Color>) -> Self {
                Self(self.0.with_fill(fill))
            }

            /// Set the outline properties and return self for chaining
            pub fn with_outline(self, outline: ($crate::style::theme::Color, f32)) -> Self {
                Self(self.0.with_outline(outline))
            }

            /// Set underline to true and return self for chaining
            pub fn with_underline(self) -> Self {
                Self(self.0.with_underline())
            }

            /// Set strikeout to true and return self for chaining
            pub fn with_strikeout(self) -> Self {
                Self(self.0.with_strikeout())
            }

            /// Get the font size
            pub fn font_size(&self) -> f32 {
                self.0.font_size()
            }

            /// Get the font
            pub fn font(&self) -> &$crate::text::font::Font {
                self.0.font()
            }

            /// Get the fill color
            pub fn fill(&self) -> Option<$crate::style::theme::Color> {
                self.0.fill()
            }

            /// Get the outline properties
            pub fn outline(&self) -> Option<($crate::style::theme::Color, f32)> {
                self.0.outline()
            }

            /// Check if strikeout is enabled
            pub fn underline(&self) -> bool {
                self.0.underline()
            }
        }

        /// Rich text structure with plotive theme colors
        #[derive(Debug, Clone)]
        pub struct $text_struct {
            text: String,
            props: $props_struct,
            spans: Vec<(usize, usize, $opt_props_struct)>,
        }

        impl From<String> for $text_struct {
            fn from(text: String) -> Self {
                $text_struct {
                    text,
                    props: $props_struct::default(),
                    spans: Vec::new(),
                }
            }
        }

        impl From<&str> for $text_struct {
            fn from(text: &str) -> Self {
                $text_struct {
                    text: text.to_string(),
                    props: $props_struct::default(),
                    spans: Vec::new(),
                }
            }
        }

        impl From<$crate::text::ParsedRichText<$crate::style::theme::Color>> for $text_struct {
            fn from(text: $crate::text::ParsedRichText<$crate::style::theme::Color>) -> Self {
                $text_struct {
                    text: text.text,
                    props: $props_struct::default(),
                    spans: text.prop_spans,
                }
            }
        }

        impl $text_struct {
            /// Set the base properties and return self for chaining
            pub fn with_props(self, props: $props_struct) -> Self {
                Self { props, ..self }
            }

            /// Set the spans and return self for chaining
            pub fn with_spans(self, spans: Vec<(usize, usize, $opt_props_struct)>) -> Self {
                Self { spans, ..self }
            }

            /// Get the text content
            pub fn text(&self) -> &str {
                &self.text
            }

            /// Get the base properties
            pub fn props(&self) -> &$props_struct {
                &self.props
            }

            /// Get the spans
            pub fn spans(&self) -> &[(usize, usize, $opt_props_struct)] {
                &self.spans
            }

            pub(crate) fn to_rich_text(
                &self,
                layout: $crate::text::rich::Layout,
                db: &$crate::text::fontdb::Database,
            ) -> std::result::Result<
                $crate::text::RichText<$crate::style::theme::Color>,
                $crate::text::Error,
            > {
                let mut builder =
                    $crate::text::RichTextBuilder::new(self.text.clone(), self.props.0.clone())
                        .with_layout(layout);
                for (start, end, props) in &self.spans {
                    builder.add_span(*start, *end, props.clone());
                }
                builder.done(db)
            }
        }
    };
}

pub(self) use define_rich_text_structs;
