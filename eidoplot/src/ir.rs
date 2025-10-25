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

// Structs defined with this macro allow to use theme::Color in place of rich::Color
// in text properties.
// In addition, caller can impl a specific Default for the $props_struct.
macro_rules! define_rich_text_structs {
    ($text_struct:ident, $props_struct:ident, $opt_props_struct:ident) => {
        /// A set of properties to be applied to a text span.
        /// If a property is `None`, value is inherited from the parent span.
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct $opt_props_struct {
            pub font_family: Option<Vec<eidoplot_text::font::Family>>,
            pub font_weight: Option<eidoplot_text::font::Weight>,
            pub font_width: Option<eidoplot_text::font::Width>,
            pub font_style: Option<eidoplot_text::font::Style>,
            pub font_size: Option<f32>,
            pub fill: Option<$crate::style::theme::Fill>,
            pub outline: Option<$crate::style::theme::Line>,
            pub underline: Option<bool>,
            pub strikeout: Option<bool>,
        }

        impl $opt_props_struct {
            fn to_rich(&self) -> eidoplot_text::rich::TextOptProps<$crate::style::theme::Color> {
                let fill = self
                    .fill
                    .as_ref()
                    .map(|f| crate::ir::paint_to_rich_fill(&f));
                let outline = self
                    .outline
                    .as_ref()
                    .map(|l| crate::ir::stroke_to_rich_outline(&l));
                eidoplot_text::rich::TextOptProps {
                    font_family: self.font_family.clone(),
                    font_weight: self.font_weight,
                    font_width: self.font_width,
                    font_style: self.font_style,
                    font_size: self.font_size,
                    fill,
                    stroke: outline,
                    underline: self.underline,
                    strikeout: self.strikeout,
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct $props_struct {
            font_size: f32,
            font: eidoplot_text::font::Font,
            fill: Option<$crate::style::theme::Fill>,
            outline: Option<$crate::style::theme::Line>,
            underline: bool,
            strikeout: bool,
        }

        impl $props_struct {
            fn new(font_size: f32) -> Self {
                Self {
                    font_size,
                    font: $crate::style::defaults::FONT_FAMILY.parse().unwrap(),
                    fill: Some($crate::style::theme::Col::Foreground.into()),
                    outline: None,
                    underline: false,
                    strikeout: false,
                }
            }
            pub fn with_font(mut self, font: eidoplot_text::font::Font) -> Self {
                self.font = font;
                self
            }

            pub fn with_fill(mut self, fill: Option<$crate::style::theme::Fill>) -> Self {
                self.fill = fill;
                self
            }

            pub fn with_outline(mut self, outline: $crate::style::theme::Line) -> Self {
                self.outline = Some(outline);
                self
            }

            pub fn with_underline(mut self) -> Self {
                self.underline = true;
                self
            }

            pub fn font_size(&self) -> f32 {
                self.font_size
            }

            pub fn font(&self) -> &eidoplot_text::font::Font {
                &self.font
            }

            pub fn fill(&self) -> Option<&$crate::style::theme::Fill> {
                self.fill.as_ref()
            }

            pub fn outline(&self) -> Option<&$crate::style::theme::Line> {
                self.outline.as_ref()
            }

            pub fn underline(&self) -> bool {
                self.underline
            }

            fn to_rich(&self) -> eidoplot_text::rich::TextProps<$crate::style::theme::Color> {
                let fill = self
                    .fill
                    .as_ref()
                    .map(|f| crate::ir::paint_to_rich_fill(&f));
                let outline = self
                    .outline
                    .as_ref()
                    .map(|l| crate::ir::stroke_to_rich_outline(&l));
                let mut props = eidoplot_text::rich::TextProps::new(self.font_size)
                    .with_font(self.font.clone())
                    .with_fill(fill);
                if let Some(outline) = outline {
                    props = props.with_outline(outline);
                }
                if self.underline {
                    props = props.with_underline();
                }
                if self.strikeout {
                    props = props.with_strikeout();
                }
                props
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
                    props: TitleProps::default(),
                    spans: Vec::new(),
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
            ) -> std::result::Result<eidoplot_text::rich::RichText, eidoplot_text::Error>
            where
                R: $crate::style::ResolveColor<$crate::style::theme::Color>,
            {
                let mut builder = eidoplot_text::rich::RichTextBuilder::new(
                    self.text.clone(),
                    self.props.to_rich(),
                )
                .with_layout(layout);
                for (start, end, props) in &self.spans {
                    builder.add_span(*start, *end, props.to_rich());
                }
                builder.done(db, rc)
            }
        }
    };
}

pub(self) use define_rich_text_structs;

use crate::style;

fn paint_to_rich_fill(fill: &style::theme::Fill) -> style::theme::Color {
    match fill {
        style::Fill::Solid { color, .. } => color.clone(),
    }
}

fn stroke_to_rich_outline(stroke: &style::theme::Line) -> (style::theme::Color, f32) {
    assert!(
        matches!(stroke.pattern, style::LinePattern::Solid),
        "Only solid outline is supported"
    );
    (stroke.color, stroke.width)
}
