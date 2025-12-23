use eidoplot::{drawing, style};
use iced::advanced::graphics::geometry;
use iced::advanced::{Layout, Widget, layout, mouse, renderer, widget};
use iced::{Element, Length, Size};

use crate::surface;

pub fn figure<'a, Theme>(fig: &'a drawing::Figure) -> Figure<'a, Theme>
where
    Theme: Catalog,
{
    Figure::new(fig)
}

#[derive(Debug)]
pub struct Figure<'a, Theme = iced::Theme>
where
    Theme: Catalog,
{
    fig: &'a drawing::Figure,
    width: Length,
    height: Length,
    scale: f32,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Figure<'a, Theme>
where
    Theme: Catalog,
{
    pub fn new(fig: &'a drawing::Figure) -> Self {
        Figure {
            fig,
            width: Length::Shrink,
            height: Length::Shrink,
            scale: 1.0,
            class: Theme::default(),
        }
    }

    /// Set the width of the [`Figure`]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the height of the [`Figure`]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Set the scale of the [`Figure`]
    pub fn scale(mut self, scale: impl Into<f32>) -> Self {
        self.scale = scale.into();
        self
    }

    /// Sets the style of the [`Figure`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Figure<'a, Theme>
where
    Renderer: iced::advanced::graphics::geometry::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let fig_size = self.fig.size();
        let intrinsic_size = Size {
            width: fig_size.width() * self.scale,
            height: fig_size.height() * self.scale,
        };
        let size = limits.resolve(self.width, self.height, intrinsic_size);
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let style = theme.style(&self.class);
        let theme = style.theme.unwrap_or_default();
        let bounds = layout.bounds();
        let frame = renderer.new_frame(bounds);
        let mut surface = surface::IcedSurface::new(frame, bounds);
        if self.fig.draw(&mut surface, &theme).is_ok() {
            let geometry = surface.into_geometry();
            renderer.draw_geometry(geometry);
        };
    }
}

/// The appearance of the figure.
#[derive(Debug, Clone, Default)]
pub struct Style {
    /// The [`Theme`](eidoplot::style::Theme) of the text.
    ///
    /// The default, `None`, means using the standard theme.
    pub theme: Option<style::Theme>,
}

impl From<Option<style::Theme>> for Style {
    fn from(theme: Option<style::Theme>) -> Self {
        Style { theme }
    }
}

/// The theme catalog of a [`Figure`].
pub trait Catalog: Sized {
    /// The item class of this [`Catalog`].
    type Class<'a>;

    /// The default class produced by this [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, item: &Self::Class<'_>) -> Style;
}

pub fn map_theme(theme: &iced::Theme) -> Option<style::Theme> {
    match theme {
        iced::Theme::Light => Some(style::theme::standard_light()),
        iced::Theme::Dark => Some(style::theme::standard_dark()),
        iced::Theme::CatppuccinLatte => Some(style::catppuccin::latte()),
        iced::Theme::CatppuccinFrappe => Some(style::catppuccin::frappe()),
        iced::Theme::CatppuccinMacchiato => Some(style::catppuccin::macchiato()),
        iced::Theme::CatppuccinMocha => Some(style::catppuccin::mocha()),
        iced::Theme::SolarizedDark => Some(style::theme::dark(style::series::palettes::pastel())),
        iced::Theme::Dracula => Some(style::theme::standard_dark()),
        iced::Theme::GruvboxDark => Some(style::theme::standard_dark()),
        iced::Theme::TokyoNight => Some(style::theme::standard_dark()),
        _ => None,
    }
}

/// A styling function for a [`Figure`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme| {
            let eplot_theme = map_theme(theme);
            Style { theme: eplot_theme }
        })
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

impl<'a, Message, Theme, Renderer> From<Figure<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer,
{
    fn from(figure: Figure<'a, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(figure)
    }
}
