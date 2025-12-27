//! Module that provides the "show" functionality using the `iced` GUI library
//! Requires the `iced` feature to be enabled

use std::sync::Arc;

use eidoplot::Drawing;
use eidoplot::data;
use eidoplot::drawing::zoom;
use eidoplot::fontdb;
use eidoplot::ir;
use eidoplot::style::CustomStyle;
use eidoplot::{drawing, geom};
use iced::Length;
use iced::widget::column;

use crate::figure::figure;
use crate::toolbar;

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
        D: data::Source + 'static;
}

impl Show for ir::Figure {
    fn show<D>(
        &self,
        data_source: Arc<D>,
        fontdb: Arc<fontdb::Database>,
        style: Option<CustomStyle>,
    ) -> iced::Result
    where
        D: data::Source + 'static,
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
    Toolbar(toolbar::Message),

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
}

struct ShowWindow<D> {
    figure: drawing::Figure,
    style: Option<CustomStyle>,
    tb_state: toolbar::State,
    interaction: Interaction,
    home_view: zoom::FigureView,
    data_source: Arc<D>,
    fontdb: Arc<fontdb::Database>,
}

impl<D> ShowWindow<D>
where
    D: data::Source + 'static,
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
            style,
            tb_state: Default::default(),
            interaction: Interaction::None,
            home_view,
            data_source,
            fontdb,
        }
    }

    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Toolbar(msg) => match msg {
                toolbar::Message::Home => {
                    self.figure
                        .apply_view(&self.home_view, &*self.data_source, Some(&*self.fontdb))
                        .expect("Failed to apply home view");
                    self.tb_state.at_home = true;
                    self.interaction = Interaction::None;
                }
                toolbar::Message::Zoom => {
                    // Toggle zoom interaction
                    match &self.interaction {
                        Interaction::ZoomEnabled | Interaction::ZoomDragging { .. } => {
                            self.interaction = Interaction::None;
                            self.tb_state.zooming = false;
                        }
                        _ => {
                            self.interaction = Interaction::ZoomEnabled;
                            self.tb_state.zooming = true;
                        }
                    };
                }
            },
            Message::FigureMouseMove(point) => {
                let hit = self.figure.hit_test(point);

                let status = hit
                    .as_ref()
                    .map(|h| format!("X = {} | Y = {}", h.x_coords, h.y_coords));
                self.tb_state.status = status;

                match (&mut self.interaction, &hit) {
                    (Interaction::ZoomDragging { idx: plot, end, .. }, Some(hit)) => {
                        if *plot == hit.idx {
                            *end = point;
                        }
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
                            self.tb_state.at_home = false;
                        }
                    }
                    self.interaction = Interaction::ZoomEnabled;
                }
                _ => {
                    self.interaction = Interaction::None;
                    self.tb_state.zooming = false;
                }
            },
            Message::Event(iced::event::Event::Mouse(ev)) => match ev {
                iced::mouse::Event::CursorLeft => {
                    self.tb_state.status = None;
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
                        self.tb_state.zooming = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let fig = figure(&self.figure)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_mouse_move(Message::FigureMouseMove)
            .on_mouse_press(Message::FigureMousePress)
            .on_mouse_release(Message::FigureMouseRelease);

        let fig = if let Some(style) = &self.style {
            fig.style(|_| style.clone())
        } else {
            fig
        };

        let fig = match &self.interaction {
            Interaction::ZoomDragging { start, end, .. } => fig.zoom_rect(*start, *end),
            _ => fig,
        };

        let toolbar = toolbar::view(&self.tb_state).map(Message::Toolbar);

        column![fig, toolbar].into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(Message::Event)
    }
}
