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
pub use plot::{Plot, PlotLegend, Subplots};
pub use series::{DataCol, Series};

// Structs defined with this macro use theme::Color for the generic color of rich properties
// Caller must impl a specific Default for the $props_struct.
macro_rules! define_rich_text_structs {
    ($text_struct:ident, $props_struct:ident, $opt_props_struct:ident) => {
        pub type $opt_props_struct = eidoplot_text::rich::TextOptProps<$crate::style::theme::Color>;

        #[derive(Debug, Clone)]
        pub struct $props_struct(eidoplot_text::rich::TextProps<$crate::style::theme::Color>);

        impl $props_struct {
            fn new(font_size: f32) -> Self {
                Self(
                    eidoplot_text::rich::TextProps::new(font_size)
                        .with_font($crate::style::defaults::FONT_FAMILY.parse().unwrap()),
                )
            }
            pub fn with_font(self, font: eidoplot_text::font::Font) -> Self {
                Self(self.0.with_font(font))
            }

            pub fn with_fill(self, fill: Option<$crate::style::theme::Color>) -> Self {
                Self(self.0.with_fill(fill))
            }

            pub fn with_outline(self, outline: ($crate::style::theme::Color, f32)) -> Self {
                Self(self.0.with_outline(outline))
            }

            pub fn with_underline(self) -> Self {
                Self(self.0.with_underline())
            }

            pub fn with_strikeout(self) -> Self {
                Self(self.0.with_strikeout())
            }

            pub fn font_size(&self) -> f32 {
                self.0.font_size()
            }

            pub fn font(&self) -> &eidoplot_text::font::Font {
                self.0.font()
            }

            pub fn fill(&self) -> Option<$crate::style::theme::Color> {
                self.0.fill()
            }

            pub fn outline(&self) -> Option<($crate::style::theme::Color, f32)> {
                self.0.outline()
            }

            pub fn underline(&self) -> bool {
                self.0.underline()
            }
        }

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

        impl From<eidoplot_text::ParsedRichText<$crate::style::theme::Color>> for $text_struct {
            fn from(text: eidoplot_text::ParsedRichText<$crate::style::theme::Color>) -> Self {
                $text_struct {
                    text: text.text,
                    props: $props_struct::default(),
                    spans: text.prop_spans,
                }
            }
        }

        impl $text_struct {
            pub fn with_props(self, props: $props_struct) -> Self {
                Self { props, ..self }
            }

            pub fn with_spans(self, spans: Vec<(usize, usize, $opt_props_struct)>) -> Self {
                Self { spans, ..self }
            }

            pub fn text(&self) -> &str {
                &self.text
            }

            pub fn props(&self) -> &$props_struct {
                &self.props
            }

            pub fn spans(&self) -> &[(usize, usize, $opt_props_struct)] {
                &self.spans
            }

            pub(crate) fn to_rich_text<R>(
                &self,
                layout: eidoplot_text::rich::Layout,
                db: &eidoplot_text::fontdb::Database,
                rc: &R,
            ) -> std::result::Result<eidoplot_text::RichText, eidoplot_text::Error>
            where
                R: $crate::style::ResolveColor<$crate::style::theme::Color>,
            {
                let mut builder =
                    eidoplot_text::RichTextBuilder::new(self.text.clone(), self.props.0.clone())
                        .with_layout(layout);
                for (start, end, props) in &self.spans {
                    builder.add_span(*start, *end, props.clone());
                }
                builder.done(db, rc)
            }
        }
    };
}

pub(self) use define_rich_text_structs;
