//! Module that provides the "show" functionality using the `iced` GUI library

use std::sync::Arc;

use iced::widget::{button, column, mouse_area, row, space, text};
use iced::{Alignment, Length, mouse};
use iced_font_awesome::{fa_icon, fa_icon_solid};
use plotive::drawing::zoom;
use plotive::style::{self, BuiltinStyle, CustomStyle};
use plotive::{Drawing, Style, data, drawing, fontdb, geom, des};

use crate::figure::figure;

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show<D, T, P>(
        self,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<Style<T, P>>,
    ) -> iced::Result
    where
        D: data::Source + ?Sized + 'static,
        T: style::Theme,
        P: style::series::Palette;
}

impl Show for des::Figure {
    fn show<D, T, P>(
        self,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<Style<T, P>>,
    ) -> iced::Result
    where
        D: data::Source + ?Sized + 'static,
        T: style::Theme,
        P: style::series::Palette,
    {
        let fig = self
            .prepare(&*data_source, Some(&*fontdb))
            .expect("Failed to prepare figure");
        show_app(fig, data_source, fontdb, style.map(|s| s.to_custom()))
    }
}

impl Show for drawing::Figure {
    fn show<D, T, P>(
        self,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<Style<T, P>>,
    ) -> iced::Result
    where
        D: data::Source + ?Sized + 'static,
        T: style::Theme,
        P: style::series::Palette,
    {
        show_app(self, data_source, fontdb, style.map(|s| s.to_custom()))
    }
}

fn show_app<D>(
    fig: drawing::Figure,
    data_source: Arc<D>,
    fontdb: Arc<fontdb::Database>,
    style: Option<CustomStyle>,
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
            (
                ShowWindow::new(fig, data_source, fontdb, style),
                iced::Task::none(),
            )
        },
        ShowWindow::update,
        ShowWindow::view,
    )
    // subscribe to key events
    .subscription(ShowWindow::subscription)
    .run()
}

#[derive(Debug, Clone)]
enum Message {
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

struct ShowWindow<D: ?Sized> {
    figure: drawing::Figure,
    home_view: zoom::FigureView,
    data_source: Arc<D>,
    fontdb: Arc<fontdb::Database>,
    style: Option<CustomStyle>,
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

impl<D> ShowWindow<D>
where
    D: data::Source + ?Sized + 'static,
{
    fn new(
        figure: drawing::Figure,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<CustomStyle>,
    ) -> Self {
        let home_view = figure.view();
        Self {
            figure,
            home_view,
            data_source,
            fontdb,
            style,
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

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }

    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::GoHome => {
                self.figure
                    .apply_view(&self.home_view, &*self.data_source, Some(&*self.fontdb))
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
                let hit = self.figure.hit_test(point);
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
                        let view = self.figure.plot_view(*idx).expect("Plot index invalid");
                        let rect = view.rect().translate(-delta_x, -delta_y);
                        let zoom = zoom::Zoom::new(rect);
                        self.figure
                            .apply_zoom(*idx, &zoom, &*self.data_source, Some(&*self.fontdb))
                            .expect("Failed to apply pan");
                        self.at_home = false;
                    }
                    _ => {}
                }

                if let Some((plot_idx, last)) = self.middle_but_drag.as_mut() {
                    let delta_x = point.x - last.x;
                    let delta_y = point.y - last.y;
                    *last = point;
                    let view = self
                        .figure
                        .plot_view(*plot_idx)
                        .expect("Plot index invalid");
                    let rect = view.rect().translate(-delta_x, -delta_y);
                    let zoom = zoom::Zoom::new(rect);
                    self.figure
                        .apply_zoom(*plot_idx, &zoom, &*self.data_source, Some(&*self.fontdb))
                        .expect("Failed to apply pan");
                    self.at_home = false;
                }
            }
            Message::FigureMousePress(point, mouse::Button::Left) => {
                let hit = self.figure.hit_test_idx(point);
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
                    let hit = self.figure.hit_test_idx(point);
                    if let Some(hit_plot_idx) = hit {
                        let rect = geom::Rect::from_corners(*start, *end);
                        if *idx == hit_plot_idx {
                            let zoom = zoom::Zoom::new(rect);
                            self.figure
                                .apply_zoom(*idx, &zoom, &*self.data_source, Some(&*self.fontdb))
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
                let hit = self.figure.hit_test_idx(point);
                if let Some(plot_idx) = hit {
                    self.middle_but_drag = Some((plot_idx, point));
                }
            }
            Message::FigureMouseRelease(_point, mouse::Button::Middle) => {
                self.middle_but_drag = None;
            }
            Message::FigureMouseWheel(point, delta) => {
                let hit = self.figure.hit_test_idx(point);
                if let Some(plot_idx) = hit {
                    let view = self.figure.plot_view(plot_idx).expect("Plot index invalid");
                    let scale_factor = (1.0 + delta * 0.1).max(0.1);
                    let rect = view.rect().scale_about(point, scale_factor);
                    let zoom = zoom::Zoom::new(rect);
                    self.figure
                        .apply_zoom(plot_idx, &zoom, &*self.data_source, Some(&*self.fontdb))
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
                        BuiltinStyle::default().to_custom()
                    };
                    let scale = self.fig_scale;
                    self.figure
                        .save_png(path, plotive_pxl::DrawingParams { style, scale })
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
                        BuiltinStyle::default().to_custom()
                    };
                    let scale = self.fig_scale;
                    self.figure
                        .save_svg(path, plotive_svg::DrawingParams { style, scale })
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
                    BuiltinStyle::default().to_custom()
                };
                let scale = self.fig_scale;
                let pixmap = self
                    .figure
                    .to_pixmap(plotive_pxl::DrawingParams { style, scale })
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
            _ => {}
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        column![self.figure_view(), self.toolbar_view()].into()
    }

    fn figure_view(&self) -> iced::Element<'_, Message> {
        let mut fig = figure(&self.figure)
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

    fn toolbar_view(&self) -> iced::Element<'_, Message> {
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
            .on_press_maybe((!self.at_home).then_some(Message::GoHome));
        let zoom_button =
            button(fa_icon_solid(FA_ZOOM).size(ICON_SZ)).on_press(Message::EnableZoom);
        let zoom_button = if zooming {
            zoom_button.style(button::secondary)
        } else {
            zoom_button.style(button::primary)
        };
        let pan_button = button(fa_icon_solid(FA_PAN).size(ICON_SZ)).on_press(Message::EnablePan);
        let pan_button = if panning {
            pan_button.style(button::secondary)
        } else {
            pan_button.style(button::primary)
        };

        let (x, y) = if let Some((x, y)) = &self.tb_status {
            (x.as_str(), y.as_str())
        } else {
            ("", "")
        };

        const TEXT_SZ: u32 = 12;
        let status_txt = column![text(x).size(TEXT_SZ), text(y).size(TEXT_SZ),]
            .padding(4)
            .spacing(5);

        let convert_png = button(
            row![fa_icon("file-image"), text("PNG").size(TEXT_SZ)]
                .align_y(Alignment::Center)
                .spacing(5),
        )
        .on_press(Message::ExportPng);
        let convert_svg = button(
            row![fa_icon("file-image"), text("SVG").size(TEXT_SZ)]
                .align_y(Alignment::Center)
                .spacing(5),
        )
        .on_press(Message::ExportSvg);

        let mut tb = row![
            home_button,
            zoom_button,
            pan_button,
            status_txt,
            space::horizontal(),
            convert_png,
            convert_svg,
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Shrink)
        .spacing(10)
        .padding(5);

        #[cfg(feature = "clipboard")]
        {
            let icon = if self.ack_clipboard {
                fa_icon_solid("check")
            } else {
                fa_icon("clipboard")
            };

            let convert_clipboard = button(icon).on_press(Message::ExportClipboard);
            tb = tb.push(convert_clipboard);
        }

        tb.into()
    }
}
