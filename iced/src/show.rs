//! Module that provides the "show" functionality using the `iced` GUI library

use core::fmt;
use std::sync::Arc;

use iced::widget::{button, column, mouse_area, row, space, text};
use iced::{Alignment, Length, mouse};
use iced_font_awesome::{fa_icon, fa_icon_solid};
use plotive::drawing::zoom;
use plotive::{Prepare, data, des, drawing, fontdb, geom};

use crate::figure::figure;

#[derive(Clone, Copy)]
pub struct Commands(u16);

impl Commands {
    pub fn all() -> Self {
        Self(0xffff)
    }

    pub fn none() -> Self {
        Self(0)
    }

    const VIEW: u16 = 0x1;
    const PNG: u16 = 0x2;
    const SVG: u16 = 0x4;
    #[cfg(feature = "clipboard")]
    const CLIPBOARD: u16 = 0x8;
    #[cfg(feature = "data-csv")]
    const CSV: u16 = 0x10;

    pub fn has_view(&self) -> bool {
        (self.0 & Self::VIEW) != 0
    }

    pub fn with_view(mut self) -> Self {
        self.0 |= Self::VIEW;
        self
    }

    pub fn without_view(mut self) -> Self {
        self.0 &= !Self::VIEW;
        self
    }

    pub fn has_export_png(&self) -> bool {
        (self.0 & Self::PNG) != 0
    }

    pub fn with_export_png(mut self) -> Self {
        self.0 |= Self::PNG;
        self
    }

    pub fn without_export_png(mut self) -> Self {
        self.0 &= !Self::PNG;
        self
    }

    pub fn has_export_svg(&self) -> bool {
        (self.0 & Self::SVG) != 0
    }

    pub fn with_export_svg(mut self) -> Self {
        self.0 |= Self::SVG;
        self
    }

    pub fn without_export_svg(mut self) -> Self {
        self.0 &= !Self::SVG;
        self
    }

    #[cfg(feature = "clipboard")]
    pub fn has_export_clipboard(&self) -> bool {
        (self.0 & Self::CLIPBOARD) != 0
    }

    #[cfg(feature = "clipboard")]
    pub fn with_export_clipboard(mut self) -> Self {
        self.0 |= Self::CLIPBOARD;
        self
    }

    #[cfg(feature = "clipboard")]
    pub fn without_export_clipboard(mut self) -> Self {
        self.0 &= !Self::CLIPBOARD;
        self
    }

    #[cfg(feature = "data-csv")]
    pub fn has_export_csv(&self) -> bool {
        (self.0 & Self::CSV) != 0
    }

    #[cfg(feature = "data-csv")]
    pub fn with_export_csv(mut self) -> Self {
        self.0 |= Self::CSV;
        self
    }

    #[cfg(feature = "data-csv")]
    pub fn without_export_csv(mut self) -> Self {
        self.0 &= !Self::CSV;
        self
    }
}

impl fmt::Debug for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut cmds = vec![];
        if self.has_view() {
            cmds.push("View");
        }
        if self.has_export_png() {
            cmds.push("ExportPng");
        }
        if self.has_export_svg() {
            cmds.push("ExportSvg");
        }
        #[cfg(feature = "clipboard")]
        if self.has_export_clipboard() {
            cmds.push("ExportClipboard");
        }
        write!(f, "Commands({})", cmds.join(" | "))
    }
}

#[derive(Debug, Clone)]
pub struct Params {
    pub style: Option<plotive::Style>,
    pub fontdb: Option<Arc<fontdb::Database>>,
    pub commands: Commands,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            style: None,
            fontdb: None,
            commands: Commands::all(),
        }
    }
}

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show<D>(self, data_source: Arc<D>, params: Params) -> iced::Result
    where
        D: data::Source + ?Sized + 'static;
}

impl Show for des::Figure {
    fn show<D>(self, data_source: Arc<D>, params: Params) -> iced::Result
    where
        D: data::Source + ?Sized + 'static,
    {
        let fontdb = params
            .fontdb
            .unwrap_or_else(|| Arc::new(plotive::bundled_font_db()));
        let fig = self
            .prepare(&*data_source, Some(&*fontdb))
            .expect("Failed to prepare figure");

        show_app(fig, data_source, fontdb, params.style)
    }
}

impl Show for drawing::PreparedFigure {
    fn show<D>(self, data_source: Arc<D>, params: Params) -> iced::Result
    where
        D: data::Source + ?Sized + 'static,
    {
        let fontdb = params
            .fontdb
            .unwrap_or_else(|| Arc::new(plotive::bundled_font_db()));

        show_app(self, data_source, fontdb, params.style)
    }
}

fn show_app<D>(
    fig: drawing::PreparedFigure,
    data_source: Arc<D>,
    fontdb: Arc<fontdb::Database>,
    style: Option<plotive::Style>,
) -> iced::Result
where
    D: data::Source + ?Sized + 'static,
{
    iced::application(
        move || {
            let fig = fig.clone();
            let data_source = data_source.clone();
            let fontdb = fontdb.clone();
            let style = style.clone();
            let mut show = FigureShow::new(fontdb, Commands::all(), None);
            show.set_figure(fig, data_source.clone());
            show.set_style(style.clone());
            (show, iced::Task::none())
        },
        FigureShow::update,
        FigureShow::view,
    )
    // subscribe to key events
    .subscription(FigureShow::subscription)
    .run()
}

/// Message type for the [`FigureShow`] controller.
#[derive(Debug, Clone)]
pub enum Message {
    GoHome,
    EnableZoom,
    EnablePan,

    FigureMousePress(geom::Point, mouse::Button),
    FigureMouseMove(geom::Point),
    FigureMouseRelease(geom::Point, mouse::Button),
    FigureMouseWheel(geom::Point, f32),
    FigureScaleChange(f32),

    Event(iced::event::Event),

    ExportPng,
    ExportSvg,
    #[cfg(feature = "clipboard")]
    ExportClipboard,
    #[cfg(feature = "clipboard")]
    ResetClipboardAck,
    #[cfg(feature = "data-csv")]
    ExportCsv,
}

#[derive(Debug, Clone, Default)]
enum Interaction {
    #[default]
    None,
    ZoomEnabled,
    ZoomDragging {
        idx: des::PlotIdx,
        start: geom::Point,
        end: geom::Point,
    },
    PanEnabled,
    PanDragging {
        idx: des::PlotIdx,
        last: geom::Point,
    },
}

/// struct gathering data that is optional in FigureShow
struct Fig<D: data::Source + ?Sized + 'static> {
    fig: drawing::PreparedFigure,
    home_view: zoom::FigureView,
    data_source: Arc<D>,
}

/// A figure show controller that manages a toolbar and a figure widget,
/// mapping events of the toolbar and the figure to [`Message`] that
/// can be routed to [`FigureShow::update`]
///
/// You can use this to integrate the functionality of the [`Show`] trait
/// into your own application.
pub struct FigureShow<D: data::Source + ?Sized + 'static> {
    fig: Option<Fig<D>>,
    no_fig_placeholder: Option<String>,
    fontdb: Arc<fontdb::Database>,
    style: Option<plotive::Style>,
    commands: Commands,
    at_home: bool,
    over_plot: bool,
    tb_status: Option<(String, String)>,
    interaction: Interaction,
    middle_but_drag: Option<(des::PlotIdx, geom::Point)>,
    fig_scale: f32,
    #[cfg(feature = "clipboard")]
    clipboard: arboard::Clipboard,
    #[cfg(feature = "clipboard")]
    ack_clipboard: bool,
}

impl<D> FigureShow<D>
where
    D: data::Source + ?Sized + 'static,
{
    pub fn new(
        fontdb: Arc<fontdb::Database>,
        commands: Commands,
        no_fig_placeholder: Option<String>,
    ) -> Self {
        Self {
            fig: None,
            no_fig_placeholder,
            fontdb,
            style: None,
            commands,
            at_home: true,
            over_plot: false,
            tb_status: None,
            interaction: Interaction::None,
            middle_but_drag: None,
            fig_scale: 1.0,
            #[cfg(feature = "clipboard")]
            clipboard: arboard::Clipboard::new().unwrap(),
            #[cfg(feature = "clipboard")]
            ack_clipboard: false,
        }
    }

    pub fn set_figure(&mut self, fig: drawing::PreparedFigure, data_source: Arc<D>) {
        let home_view = fig.view();
        self.fig = Some(Fig {
            fig,
            home_view,
            data_source,
        });
        self.at_home = true;
        self.interaction = Interaction::None;
    }

    pub fn reset_figure(&mut self) {
        self.fig = None;
        self.at_home = true;
        self.interaction = Interaction::None;
    }

    pub fn set_style(&mut self, style: Option<plotive::Style>) {
        self.style = style;
    }

    pub fn figure(&self) -> Option<&drawing::PreparedFigure> {
        self.fig.as_ref().map(|f| &f.fig)
    }

    pub fn data_source(&self) -> Option<&Arc<D>> {
        self.fig.as_ref().map(|f| &f.data_source)
    }

    pub fn style(&self) -> Option<&plotive::Style> {
        self.style.as_ref()
    }

    pub fn fontdb(&self) -> &Arc<fontdb::Database> {
        &self.fontdb
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        let Some(fig) = &mut self.fig else {
            return iced::Task::none();
        };

        match msg {
            Message::GoHome => {
                fig.fig
                    .apply_view(&fig.home_view, &*fig.data_source, Some(&*self.fontdb))
                    .expect("Failed to apply home view");
                self.at_home = true;
                self.interaction = Interaction::None;
            }
            Message::EnableZoom => {
                // Toggle zoom interaction
                match &self.interaction {
                    Interaction::ZoomEnabled | Interaction::ZoomDragging { .. } => {
                        self.interaction = Interaction::None;
                    }
                    _ => {
                        self.interaction = Interaction::ZoomEnabled;
                    }
                };
            }
            Message::EnablePan => {
                // Toggle pan interaction
                match &self.interaction {
                    Interaction::PanEnabled | Interaction::PanDragging { .. } => {
                        self.interaction = Interaction::None;
                    }
                    _ => {
                        self.interaction = Interaction::PanEnabled;
                    }
                };
            }
            Message::FigureMouseMove(point) => {
                let hit = fig.fig.hit_test(point);
                self.over_plot = hit.is_some();

                let status = hit
                    .as_ref()
                    .map(|h| (format!("X = {}", h.x_coords), format!("Y = {}", h.y_coords)));
                self.tb_status = status;

                match (&mut self.interaction, &hit) {
                    (Interaction::ZoomDragging { idx: plot, end, .. }, Some(hit)) => {
                        if *plot == hit.idx {
                            *end = point;
                        }
                    }
                    // match any hit because panning can go outside plot area
                    (Interaction::PanDragging { idx, last }, _) => {
                        let delta_x = point.x - last.x;
                        let delta_y = point.y - last.y;
                        *last = point;
                        let view = fig.fig.plot_view(*idx).expect("Plot index invalid");
                        let rect = view.rect().translate(-delta_x, -delta_y);
                        let zoom = zoom::Zoom::new(rect);
                        fig.fig
                            .apply_zoom(*idx, &zoom, &*fig.data_source, Some(&*self.fontdb))
                            .expect("Failed to apply pan");
                        self.at_home = false;
                    }
                    _ => {}
                }

                if let Some((plot_idx, last)) = self.middle_but_drag.as_mut() {
                    let delta_x = point.x - last.x;
                    let delta_y = point.y - last.y;
                    *last = point;
                    let view = fig.fig.plot_view(*plot_idx).expect("Plot index invalid");
                    let rect = view.rect().translate(-delta_x, -delta_y);
                    let zoom = zoom::Zoom::new(rect);
                    fig.fig
                        .apply_zoom(*plot_idx, &zoom, &*fig.data_source, Some(&*self.fontdb))
                        .expect("Failed to apply pan");
                    self.at_home = false;
                }
            }
            Message::FigureMousePress(point, mouse::Button::Left) => {
                let hit = fig.fig.hit_test_idx(point);
                match (&self.interaction, hit) {
                    (Interaction::ZoomEnabled, Some(plot)) => {
                        self.interaction = Interaction::ZoomDragging {
                            idx: plot,
                            start: point,
                            end: point,
                        };
                    }
                    (Interaction::PanEnabled, Some(plot)) => {
                        self.interaction = Interaction::PanDragging {
                            idx: plot,
                            last: point,
                        };
                    }
                    _ => {}
                }
            }
            Message::FigureMouseRelease(point, mouse::Button::Left) => match &self.interaction {
                Interaction::ZoomDragging { idx, start, end } => {
                    let hit = fig.fig.hit_test_idx(point);
                    if let Some(hit_plot_idx) = hit {
                        let rect = geom::Rect::from_corners(*start, *end);
                        if *idx == hit_plot_idx {
                            let zoom = zoom::Zoom::new(rect);
                            fig.fig
                                .apply_zoom(*idx, &zoom, &*fig.data_source, Some(&*self.fontdb))
                                .expect("Failed to apply zoom");
                            self.at_home = false;
                        }
                    }
                    self.interaction = Interaction::ZoomEnabled;
                }
                Interaction::PanDragging { .. } => {
                    self.interaction = Interaction::PanEnabled;
                }
                _ => {
                    self.interaction = Interaction::None;
                }
            },
            Message::FigureMousePress(point, mouse::Button::Middle) => {
                let hit = fig.fig.hit_test_idx(point);
                if let Some(plot_idx) = hit {
                    self.middle_but_drag = Some((plot_idx, point));
                }
            }
            Message::FigureMouseRelease(_point, mouse::Button::Middle) => {
                self.middle_but_drag = None;
            }
            Message::FigureMouseWheel(point, delta) => {
                let hit = fig.fig.hit_test_idx(point);
                if let Some(plot_idx) = hit {
                    let view = fig.fig.plot_view(plot_idx).expect("Plot index invalid");
                    let scale_factor = (1.0 + delta * 0.1).max(0.1);
                    let rect = view.rect().scale_about(point, scale_factor);
                    let zoom = zoom::Zoom::new(rect);
                    fig.fig
                        .apply_zoom(plot_idx, &zoom, &*fig.data_source, Some(&*self.fontdb))
                        .expect("Failed to apply zoom");
                    self.at_home = false;
                }
            }
            Message::Event(iced::event::Event::Mouse(ev)) => match ev {
                iced::mouse::Event::CursorLeft => {
                    self.tb_status = None;
                }
                _ => {}
            },
            Message::Event(iced::event::Event::Keyboard(ev)) => {
                use iced::keyboard::{self, key};
                match ev {
                    keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key::Named::Escape),
                        ..
                    } => {
                        self.interaction = Interaction::None;
                    }
                    _ => {}
                }
            }
            Message::FigureScaleChange(scale) => {
                self.fig_scale = scale;
            }
            Message::ExportPng => {
                use plotive_pxl::SavePng;
                let filename = rfd::FileDialog::new()
                    .set_title("Save figure as PNG")
                    .add_filter("PNG Image", &["png"])
                    .set_file_name("figure.png")
                    .save_file();
                if let Some(path) = filename {
                    let style = if let Some(style) = &self.style {
                        style.clone()
                    } else {
                        plotive::Style::default()
                    };
                    let scale = self.fig_scale;
                    fig.fig
                        .save_png(path, plotive_pxl::Params { style, scale })
                        .unwrap();
                }
            }
            Message::ExportSvg => {
                use plotive_svg::SaveSvg;
                let filename = rfd::FileDialog::new()
                    .set_title("Save figure as SVG")
                    .add_filter("SVG Image", &["svg"])
                    .set_file_name("figure.svg")
                    .save_file();
                if let Some(path) = filename {
                    let style = if let Some(style) = &self.style {
                        style.clone()
                    } else {
                        plotive::Style::default()
                    };
                    let scale = self.fig_scale;
                    fig.fig
                        .save_svg(path, plotive_svg::Params { style, scale })
                        .unwrap();
                }
            }
            #[cfg(feature = "clipboard")]
            Message::ExportClipboard => {
                use std::borrow::Cow;

                use plotive_pxl::ToPixmap;

                let style = if let Some(style) = &self.style {
                    style.clone()
                } else {
                    plotive::Style::default()
                };
                let scale = self.fig_scale;
                let pixmap = fig
                    .fig
                    .to_pixmap(plotive_pxl::Params { style, scale })
                    .unwrap();
                self.clipboard
                    .set_image(arboard::ImageData {
                        width: pixmap.width() as usize,
                        height: pixmap.height() as usize,
                        bytes: Cow::Borrowed(pixmap.data()),
                    })
                    .unwrap();
                self.ack_clipboard = true;
                return iced::Task::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    },
                    |_| Message::ResetClipboardAck,
                );
            }
            #[cfg(feature = "clipboard")]
            Message::ResetClipboardAck => {
                if self.ack_clipboard {
                    self.ack_clipboard = false;
                }
            }
            #[cfg(feature = "data-csv")]
            Message::ExportCsv => {
                let filename = rfd::FileDialog::new()
                    .set_title("Save figure data as CSV")
                    .add_filter("CSV File", &["csv"])
                    .set_file_name("figure_data.csv")
                    .save_file();
                if let Some(path) = filename {
                    let mut file = std::fs::File::create(path).unwrap();
                    data::csv::export_data_source(&mut file, &*fig.data_source, Default::default())
                        .expect("Failed to export figure data to CSV");
                }
            }
            _ => {}
        }
        iced::Task::none()
    }

    /// Create the view for both figure and toolbar, stacked in a column with figure above toolbar.
    pub fn view(&self) -> iced::Element<'_, Message> {
        column![self.figure_view(), self.toolbar_view()].into()
    }

    /// Create the view for the figure
    pub fn figure_view(&self) -> iced::Element<'_, Message> {
        let Some(fig) = &self.fig else {
            return text(self.no_fig_placeholder.clone().unwrap_or_else(String::new))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into();
        };

        let mut fig = figure(&fig.fig)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_mouse_move(Message::FigureMouseMove)
            .on_mouse_press(Message::FigureMousePress)
            .on_mouse_release(Message::FigureMouseRelease)
            .on_mouse_wheel(Message::FigureMouseWheel)
            .on_scale_change(Message::FigureScaleChange);

        if let Some(style) = &self.style {
            fig = fig.style(|_| style.clone());
        }

        if let Interaction::ZoomDragging { start, end, .. } = &self.interaction {
            fig = fig.zoom_rect(*start, *end);
        }

        // Wrap with mouse_area to control cursor
        let interaction = match self.interaction {
            Interaction::PanEnabled if self.over_plot => iced::mouse::Interaction::Grabbing,
            Interaction::PanDragging { .. } => iced::mouse::Interaction::Grabbing,
            _ => {
                if self.over_plot {
                    iced::mouse::Interaction::Crosshair
                } else {
                    iced::mouse::Interaction::default()
                }
            }
        };

        mouse_area(fig).interaction(interaction).into()
    }

    /// Create a view for the toolbar
    pub fn toolbar_view(&self) -> iced::Element<'_, Message> {
        let has_fig = self.fig.is_some();

        let mut toolbar = row![];

        if self.commands.has_view() {
            let zooming = matches!(
                self.interaction,
                Interaction::ZoomEnabled | Interaction::ZoomDragging { .. }
            );
            let panning = matches!(
                self.interaction,
                Interaction::PanEnabled | Interaction::PanDragging { .. }
            );

            const ICON_SZ: u16 = 24;
            const FA_HOME: &str = "down-left-and-up-right-to-center";
            const FA_ZOOM: &str = "expand";
            const FA_PAN: &str = "arrows-up-down-left-right";

            let home_button = button(fa_icon_solid(FA_HOME).size(ICON_SZ))
                .on_press_maybe((has_fig && !self.at_home).then_some(Message::GoHome));
            let zoom_button = button(fa_icon_solid(FA_ZOOM).size(ICON_SZ))
                .on_press_maybe(has_fig.then_some(Message::EnableZoom));
            let zoom_button = if zooming {
                zoom_button.style(button::secondary)
            } else {
                zoom_button.style(button::primary)
            };
            let pan_button = button(fa_icon_solid(FA_PAN).size(ICON_SZ))
                .on_press_maybe(has_fig.then_some(Message::EnablePan));
            let pan_button = if panning {
                pan_button.style(button::secondary)
            } else {
                pan_button.style(button::primary)
            };

            toolbar = toolbar.push(home_button).push(zoom_button).push(pan_button);
        }

        let (x, y) = if let Some((x, y)) = &self.tb_status {
            (x.as_str(), y.as_str())
        } else {
            ("", "")
        };

        const TEXT_SZ: u32 = 12;
        let status_txt = column![text(x).size(TEXT_SZ), text(y).size(TEXT_SZ),]
            .padding(4)
            .spacing(5);
        toolbar = toolbar.push(status_txt).push(space::horizontal());

        if self.commands.has_export_png() {
            let convert_png = button(
                row![fa_icon_solid("file-image"), text("PNG").size(TEXT_SZ)]
                    .align_y(Alignment::Center)
                    .spacing(5),
            )
            .on_press_maybe(has_fig.then_some(Message::ExportPng));
            toolbar = toolbar.push(convert_png);
        }

        if self.commands.has_export_svg() {
            let convert_svg = button(
                row![fa_icon_solid("file-image"), text("SVG").size(TEXT_SZ)]
                    .align_y(Alignment::Center)
                    .spacing(5),
            )
            .on_press_maybe(has_fig.then_some(Message::ExportSvg));
            toolbar = toolbar.push(convert_svg);
        }

        #[cfg(feature = "clipboard")]
        if self.commands.has_export_clipboard() {
            let icon = if self.ack_clipboard {
                fa_icon_solid("check")
            } else {
                fa_icon("clipboard")
            };

            let convert_clipboard =
                button(icon).on_press_maybe(has_fig.then_some(Message::ExportClipboard));
            toolbar = toolbar.push(convert_clipboard);
        }

        #[cfg(feature = "data-csv")]
        if self.commands.has_export_csv() {
            let convert_csv = button(fa_icon_solid("file-csv"))
                .on_press_maybe(has_fig.then_some(Message::ExportCsv));
            toolbar = toolbar.push(convert_csv);
        }

        toolbar
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(10)
            .padding(5)
            .into()
    }
}
