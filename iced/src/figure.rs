use iced::advanced::graphics::geometry;
use iced::advanced::widget::tree;
use iced::advanced::{Layout, Widget, layout, mouse, renderer, widget};
use iced::{Element, Length, Rectangle, Size};
use plotive::render::Surface;
use plotive::style::theme;
use plotive::{drawing, geom, style};

use crate::surface;

pub fn figure<'a, Message, Theme>(fig: &'a drawing::PreparedFigure) -> Figure<'a, Message, Theme>
where
    Theme: Catalog,
{
    Figure::new(fig)
}

pub struct Figure<'a, Message, Theme = iced::Theme>
where
    Theme: Catalog,
{
    fig: &'a drawing::PreparedFigure,
    width: Length,
    height: Length,
    scale: f32,
    class: Theme::Class<'a>,
    on_mouse_press: Option<Box<dyn Fn(geom::Point, mouse::Button) -> Message + 'a>>,
    on_mouse_move: Option<Box<dyn Fn(geom::Point) -> Message + 'a>>,
    on_mouse_release: Option<Box<dyn Fn(geom::Point, mouse::Button) -> Message + 'a>>,
    on_mouse_wheel: Option<Box<dyn Fn(geom::Point, f32) -> Message + 'a>>,
    on_scale_change: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    zoom_rect: Option<(geom::Point, geom::Point)>,
}

impl<'a, Theme> std::fmt::Debug for Figure<'a, Theme>
where
    Theme: Catalog,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Figure")
            .field("fig", &self.fig)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale", &self.scale)
            .finish()
    }
}

impl<'a, Message, Theme> Figure<'a, Message, Theme>
where
    Theme: Catalog,
{
    pub fn new(fig: &'a drawing::PreparedFigure) -> Self {
        Figure {
            fig,
            width: Length::Shrink,
            height: Length::Shrink,
            scale: 1.0,
            class: Theme::default(),
            on_mouse_press: None,
            on_mouse_move: None,
            on_mouse_release: None,
            on_mouse_wheel: None,
            on_scale_change: None,
            zoom_rect: None,
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
    pub fn style(mut self, style: impl Fn(&Theme) -> plotive::Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the on mouse press callback of the [`Figure`].
    #[must_use]
    pub fn on_mouse_press(
        mut self,
        callback: impl Fn(geom::Point, mouse::Button) -> Message + 'a,
    ) -> Self {
        self.on_mouse_press = Some(Box::new(callback));
        self
    }

    /// Sets the on mouse move callback of the [`Figure`].
    #[must_use]
    pub fn on_mouse_move(mut self, callback: impl Fn(geom::Point) -> Message + 'a) -> Self {
        self.on_mouse_move = Some(Box::new(callback));
        self
    }

    /// Sets the on mouse release callback of the [`Figure`].
    #[must_use]
    pub fn on_mouse_release(
        mut self,
        callback: impl Fn(geom::Point, mouse::Button) -> Message + 'a,
    ) -> Self {
        self.on_mouse_release = Some(Box::new(callback));
        self
    }

    /// Sets the on mouse wheel callback of the [`Figure`].
    #[must_use]
    pub fn on_mouse_wheel(mut self, callback: impl Fn(geom::Point, f32) -> Message + 'a) -> Self {
        self.on_mouse_wheel = Some(Box::new(callback));
        self
    }

    /// Sets the on scale change callback of the [`Figure`].
    #[must_use]
    pub fn on_scale_change(mut self, callback: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_scale_change = Some(Box::new(callback));
        self
    }

    /// Sets the zoom rectangle of the [`Figure`].
    #[must_use]
    pub fn zoom_rect(mut self, start: geom::Point, end: geom::Point) -> Self {
        self.zoom_rect = Some((start, end));
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    dragging: bool,
    mouse_pos: Option<geom::Point>,
    cached_bounds: Option<Rectangle>,
}

fn fit_transform_to_bounds(size: geom::Size, bounds: Rectangle) -> geom::Transform {
    // scale up or down to fit the size into bounds, preserving aspect ratio and centering
    let tx = bounds.x;
    let ty = bounds.y;
    let sx = bounds.width / size.width();
    let sy = bounds.height / size.height();
    let s = sx.min(sy);
    let w = size.width() * s;
    let h = size.height() * s;
    let tx = tx + (bounds.width - w) / 2.0;
    let ty = ty + (bounds.height - h) / 2.0;
    geom::Transform::from_translate(tx, ty).pre_concat(geom::Transform::from_scale(s, s))
}

fn fit_scale_to_bounds(size: geom::Size, bounds: Rectangle) -> f32 {
    // scale up or down to fit the size into bounds, preserving aspect ratio and centering
    let sx = bounds.width / size.width();
    let sy = bounds.height / size.height();
    sx.min(sy)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Figure<'a, Message, Theme>
where
    Renderer: iced::advanced::graphics::geometry::Renderer,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

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

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        if shell.is_event_captured() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();

        let bounds = layout.bounds();
        if state.cached_bounds != Some(bounds) {
            state.cached_bounds = Some(bounds);
            let size = fit_scale_to_bounds(self.fig.size(), bounds);
            if let Some(callback) = &self.on_scale_change {
                let msg = callback(size);
                shell.publish(msg);
            }
        }

        if !cursor.is_over(bounds) && !state.dragging {
            state.mouse_pos = None;
            return;
        }

        match event {
            iced::Event::Window(window_ev) => {
                if let iced::window::Event::Resized { .. } = window_ev {
                    // Clear mouse position on resize to avoid incorrect coordinates
                    state.mouse_pos = None;
                }
            }
            iced::Event::Mouse(mouse_ev) => match mouse_ev {
                mouse::Event::CursorMoved { position } => {
                    let transform = fit_transform_to_bounds(self.fig.size(), layout.bounds())
                        .invert()
                        .expect("transform without skew should be invertible");
                    let mut point = geom::Point {
                        x: position.x,
                        y: position.y,
                    };
                    transform.map_point(&mut point);
                    state.mouse_pos = Some(point);
                    if let Some(callback) = &self.on_mouse_move {
                        let msg = callback(point);
                        shell.publish(msg);
                    }
                    shell.capture_event();
                }
                mouse::Event::ButtonPressed(but) => {
                    state.dragging = true;
                    if let Some(pos) = state.mouse_pos {
                        if let Some(callback) = &self.on_mouse_press {
                            let msg = callback(pos, *but);
                            shell.publish(msg);
                        }
                    }
                    shell.capture_event();
                }
                mouse::Event::ButtonReleased(but) => {
                    state.dragging = false;
                    if let Some(pos) = state.mouse_pos {
                        if let Some(callback) = &self.on_mouse_release {
                            let msg = callback(pos, *but);
                            shell.publish(msg);
                        }
                    }
                    shell.capture_event();
                }
                mouse::Event::WheelScrolled { delta } => {
                    if let Some(pos) = state.mouse_pos {
                        if let Some(callback) = &self.on_mouse_wheel {
                            let scroll_amount = match delta {
                                mouse::ScrollDelta::Lines { y, .. } => *y,
                                mouse::ScrollDelta::Pixels { y, .. } => *y / 50.0,
                            };
                            let msg = callback(pos, scroll_amount);
                            shell.publish(msg);
                        }
                    }
                    shell.capture_event();
                }
                _ => {}
            },
            _ => {}
        }
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

        let bounds = layout.bounds();
        let transform = fit_transform_to_bounds(self.fig.size(), bounds);

        let frame = renderer.new_frame(bounds);
        let mut surface = surface::IcedSurface::new(frame, bounds, transform);

        self.fig.draw(&mut surface, &style);

        if let Some((start, end)) = self.zoom_rect {
            let rect = geom::Rect::from_corners(start, end);
            let stroke = theme::Stroke::from(theme::Col::Foreground)
                .with_pattern(style::Dash::default().into());
            let stroke = stroke.as_stroke(&style);
            let _ = surface.draw_rect(&plotive::render::Rect {
                rect,
                stroke: Some(stroke),
                fill: None,
                transform: None,
            });
        }

        for g in surface.into_geometries() {
            renderer.draw_geometry(g);
        }
    }
}

/// The theme catalog of a [`Figure`].
pub trait Catalog: Sized {
    /// The item class of this [`Catalog`].
    type Class<'a>;

    /// The default class produced by this [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`](plotive::Style) of a class with the given status.
    fn style(&self, item: &Self::Class<'_>) -> plotive::Style;
}

#[inline]
fn from_iced_color(color: iced::Color) -> plotive::ColorU8 {
    let [r, g, b, a] = color.into_rgba8();
    plotive::ColorU8::from_rgba(r, g, b, a)
}

/// Map an `iced::Theme` to an plotive theme.
pub fn map_theme(theme: &iced::Theme) -> theme::Theme {
    let pal = theme.palette();
    let back = from_iced_color(pal.background);
    let fore = from_iced_color(pal.text);
    theme::Theme::Custom(theme::ThemePalette::new_back_and_fore(back, fore))
}

/// Map an `iced::Theme` to an plotive style.
pub fn map_style(theme: &iced::Theme) -> plotive::Style {
    match theme {
        iced::Theme::CatppuccinMocha => plotive::Style::catppuccin_mocha(),
        iced::Theme::CatppuccinMacchiato => plotive::Style::catppuccin_macchiato(),
        iced::Theme::CatppuccinFrappe => plotive::Style::catppuccin_frappe(),
        iced::Theme::CatppuccinLatte => plotive::Style::catppuccin_latte(),
        _ => {
            let theme = map_theme(theme);
            let palette = if theme.is_dark() {
                style::series::Palette::Pastel
            } else {
                style::series::Palette::Standard
            };
            plotive::Style::new(theme, palette)
        }
    }
}

/// A styling function for a [`Figure`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> plotive::Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme| map_style(theme))
    }

    fn style(&self, class: &Self::Class<'_>) -> plotive::Style {
        class(self)
    }
}

impl<'a, Message, Theme, Renderer> From<Figure<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: geometry::Renderer,
{
    fn from(figure: Figure<'a, Message, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(figure)
    }
}
