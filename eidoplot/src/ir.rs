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

macro_rules! define_rich_text_structs {
    ($props_name:ident, $struct_name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $props_name(pub eidoplot_text::rich::TextProps);

        impl $props_name {
            pub fn with_font(self, font: eidoplot_text::font::Font) -> Self {
                Self (self.0.with_font(font))
            }

            pub fn with_fill(self, fill: Option<eidoplot_text::rich::Color>) -> Self {
                Self (self.0.with_fill(fill))
            }

            pub fn with_outline(self, stroke: (eidoplot_text::rich::Color, f32)) -> Self {
                Self (self.0.with_outline(stroke))
            }

            pub fn with_underline(self) -> Self {
                Self (self.0.with_underline())
            }

            pub fn with_strikeout(self) -> Self {
                Self (self.0.with_strikeout())
            }

            pub fn font_size(&self) -> f32 {
                self.0.font_size()
            }

            pub fn font(&self) -> &eidoplot_text::font::Font {
                self.0.font()
            }

            pub fn fill(&self) -> Option<eidoplot_text::rich::Color> {
                self.0.fill()
            }

            pub fn outline(&self) -> Option<(eidoplot_text::rich::Color, f32)> {
                self.0.outline()
            }

            pub fn underline(&self) -> bool {
                self.0.underline()
            }

            pub fn strikeout(&self) -> bool {
                self.0.strikeout()
            }
        }

        #[derive(Debug, Clone)]
        pub struct $struct_name {
            text: String,
            props: TitleProps,
            spans: Vec<(usize, usize, eidoplot_text::rich::TextOptProps)>,
        }

        impl From<String> for $struct_name {
            fn from(text: String) -> Self {
                Title {
                    text,
                    props: TitleProps::default(),
                    spans: Vec::new(),
                }
            }
        }

        impl $struct_name {
            pub fn with_props(self, props: $props_name) -> Self {
                Self { props, ..self }
            }

            pub fn with_spans(self, spans: Vec<(usize, usize, eidoplot_text::rich::TextOptProps)>) -> Self {
                Self { spans, ..self }
            }

            pub fn text(&self) -> &str {
                &self.text
            }

            pub fn props(&self) -> &$props_name {
                &self.props
            }

            pub fn spans(&self) -> &[(usize, usize, eidoplot_text::rich::TextOptProps)] {
                &self.spans
            }
        }
    };
}

pub(self) use define_rich_text_structs;
