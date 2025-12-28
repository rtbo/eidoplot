//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use std::sync::Arc;

use eidoplot::drawing::zoom;
use eidoplot::style::CustomStyle;
use eidoplot::{Drawing, data, drawing, fontdb, geom, ir};
use iced::widget::{button, column, mouse_area, row, space};
use iced::{Alignment, Length};
use iced_font_awesome::fa_icon_solid;

use crate::figure::figure;

/// Trait to show figures in a window
pub trait Show {
    /// Show the figure in a GUI window.
    /// This function will block the calling thread until the window is closed.
    fn show<D>(
        &self,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<CustomStyle>,
    ) -> iced::Result
    where
        D: data::Source + ?Sized + 'static;
}

impl Show for ir::Figure {
    fn show<D>(
        &self,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<CustomStyle>,
    ) -> iced::Result
    where
        D: data::Source + ?Sized + 'static,
    {
        let ir_fig = self.clone();
        iced::application(
            move || {
                let data_source = data_source.clone();
                let fontdb = fontdb.clone();
                let style = style.clone();
                let fig = ir_fig
                    .prepare(&*data_source, Some(&*fontdb))
                    .expect("Failed to prepare figure");
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
}

#[derive(Debug, Clone)]
enum Message {
    GoHome,
    EnableZoom,
    EnablePan,

    FigureMousePress(geom::Point),
    FigureMouseMove(geom::Point),
    FigureMouseRelease(geom::Point),

    Event(iced::event::Event),
}

#[derive(Debug, Clone, Default)]
enum Interaction {
    #[default]
    None,
    ZoomEnabled,
    ZoomDragging {
        idx: ir::PlotIdx,
        start: geom::Point,
        end: geom::Point,
    },
    PanEnabled,
    PanDragging {
        idx: ir::PlotIdx,
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
    tb_status: Option<String>,
    interaction: Interaction,
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
                    .map(|h| format!("X = {} | Y = {}", h.x_coords, h.y_coords));
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
            }
            Message::FigureMousePress(point) => {
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
            Message::FigureMouseRelease(point) => match &self.interaction {
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
            .on_mouse_release(Message::FigureMouseRelease);

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

        const FA_HOME: &str = "down-left-and-up-right-to-center";
        const FA_ZOOM: &str = "expand";
        const FA_PAN: &str = "arrows-up-down-left-right";

        let home_button = button(fa_icon_solid(FA_HOME))
            .on_press_maybe((!self.at_home).then_some(Message::GoHome));
        let zoom_button = button(fa_icon_solid(FA_ZOOM)).on_press(Message::EnableZoom);
        let zoom_button = if zooming {
            zoom_button.style(button::secondary)
        } else {
            zoom_button.style(button::primary)
        };
        let pan_button = button(fa_icon_solid(FA_PAN)).on_press(Message::EnablePan);
        let pan_button = if panning {
            pan_button.style(button::secondary)
        } else {
            pan_button.style(button::primary)
        };

        let status_txt = self.tb_status.as_deref().unwrap_or("");
        let status_txt = iced::widget::text(status_txt)
            .height(Length::Fill)
            .align_y(Alignment::Center);

        row![
            home_button,
            zoom_button,
            pan_button,
            space::horizontal(),
            status_txt,
        ]
        .width(Length::Fill)
        .height(Length::Shrink)
        .spacing(10)
        .padding(5)
        .into()
    }
}
