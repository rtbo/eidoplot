/*!
 * # Intermediate representation (IR) for eidoplot
 *
 * This module contains all data structures for the design of plotting figures.
 */
pub mod axis;
pub mod figure;
pub mod legend;
pub mod plot;
pub mod series;

pub use axis::Axis;
pub use figure::{FigLegend, Figure};
pub use legend::Legend;
pub use plot::{Plot, PlotLegend, PlotLine, Subplots};
pub use series::{DataCol, Series, data_src_ref};

// Structs defined with this macro use theme::Color for the generic color of rich properties
// Caller must impl a specific Default for the $props_struct.
macro_rules! define_rich_text_structs {
    ($text_struct:ident, $props_struct:ident, $opt_props_struct:ident) => {
        /// Rich text properties that can apply only some properties on a given text span
        pub type $opt_props_struct = $crate::text::rich::TextOptProps<$crate::style::theme::Color>;

        /// Rich text base properties with eidoplot theme colors
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

        /// Rich text structure with eidoplot theme colors
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
