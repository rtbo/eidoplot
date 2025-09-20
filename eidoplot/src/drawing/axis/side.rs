use eidoplot_text as text;

use crate::drawing::scale::CoordMap;
use crate::{geom, missing_params};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Side {
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Horizontal,
    Vertical,
}

impl Side {
    fn direction(&self) -> Direction {
        match self {
            Side::Bottom | Side::Top => Direction::Horizontal,
            Side::Left | Side::Right => Direction::Vertical,
        }
    }

    /// Layout options for axis title
    pub fn title_opts(&self) -> text::layout::Options {
        match self {
            Side::Bottom => text::layout::Options {
                hor_align: text::layout::HorAlign::Center,
                ver_align: text::layout::LineVerAlign::Top.into(),
                ..Default::default()
            },
            Side::Top => text::layout::Options {
                hor_align: text::layout::HorAlign::Center,
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
            Side::Left => text::layout::Options {
                hor_align: text::layout::HorAlign::Center,
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
            Side::Right => text::layout::Options {
                hor_align: text::layout::HorAlign::Center,
                ver_align: text::layout::LineVerAlign::Top.into(),
                ..Default::default()
            },
        }
    }

    pub fn title_transform(&self, shift_across: f32, rect: &geom::Rect) -> geom::Transform {
        match self {
            Side::Bottom => {
                geom::Transform::from_translate(rect.center_x(), rect.bottom() + shift_across)
            }
            Side::Top => {
                geom::Transform::from_translate(rect.center_x(), rect.top() - shift_across)
            }
            Side::Left => {
                geom::Transform::from_translate(rect.left() - shift_across, rect.center_y())
                    .pre_rotate(-90.0)
            }
            Side::Right => {
                geom::Transform::from_translate(rect.right() + shift_across, rect.center_y())
                    .pre_rotate(-90.0)
            }
        }
    }

    pub fn ticks_labels_opts(&self) -> text::layout::Options {
        match self {
            Side::Bottom => text::layout::Options {
                hor_align: text::layout::HorAlign::Center,
                ver_align: text::layout::LineVerAlign::Top.into(),
                ..Default::default()
            },
            Side::Top => text::layout::Options {
                hor_align: text::layout::HorAlign::Center,
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
            Side::Left => text::layout::Options {
                hor_align: text::layout::HorAlign::Right,
                ver_align: text::layout::LineVerAlign::Middle.into(),
                ..Default::default()
            },
            Side::Right => text::layout::Options {
                hor_align: text::layout::HorAlign::Left,
                ver_align: text::layout::LineVerAlign::Middle.into(),
                ..Default::default()
            },
        }
    }

    /// Return the transform to be applied to a tick label
    /// `pos` is the position along the axis in figure units
    /// `shift` is the distance from the axis in figure units
    /// E.g. for bottom axis, position shifts towards right, and shift shifts towards bottom
    pub fn tick_label_transform(
        &self,
        pos_along: f32,
        shift_across: f32,
        rect: &geom::Rect,
    ) -> geom::Transform {
        match self {
            Side::Bottom => geom::Transform::from_translate(
                rect.left() + pos_along,
                rect.bottom() + shift_across,
            ),
            Side::Top => {
                geom::Transform::from_translate(rect.left() + pos_along, rect.top() - shift_across)
            }
            Side::Left => geom::Transform::from_translate(
                rect.left() - shift_across,
                rect.bottom() - pos_along,
            ),
            Side::Right => geom::Transform::from_translate(
                rect.right() + shift_across,
                rect.bottom() - pos_along,
            ),
        }
    }

    pub fn annot_opts(&self) -> text::layout::Options {
        match self {
            Side::Bottom => text::layout::Options {
                hor_align: text::layout::HorAlign::Right,
                ver_align: text::layout::LineVerAlign::Top.into(),
                ..Default::default()
            },
            Side::Top => text::layout::Options {
                hor_align: text::layout::HorAlign::Right,
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
            Side::Left => text::layout::Options {
                hor_align: text::layout::HorAlign::Right,
                ver_align: text::layout::LineVerAlign::Top.into(),
                ..Default::default()
            },
            Side::Right => text::layout::Options {
                hor_align: text::layout::HorAlign::Left,
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
        }
    }

    pub fn annot_transform(&self, shift_across: f32, rect: &geom::Rect) -> geom::Transform {
        let margin = missing_params::AXIS_ANNOT_MARGIN;
        match self {
            Side::Bottom => {
                geom::Transform::from_translate(rect.right(), rect.bottom() + shift_across + margin)
            }
            Side::Top => {
                geom::Transform::from_translate(rect.right(), rect.top() - shift_across - margin)
            }
            Side::Left => {
                geom::Transform::from_translate(rect.left() - shift_across - margin, rect.top())
            }
            Side::Right => {
                geom::Transform::from_translate(rect.right() + shift_across + margin, rect.top())
            }
        }
    }

    #[allow(dead_code)]
    pub fn size_along(&self, size: &geom::Size) -> f32 {
        match self.direction() {
            Direction::Horizontal => size.width(),
            Direction::Vertical => size.height(),
        }
    }

    pub fn size_across(&self, avail_size: &geom::Size) -> f32 {
        match self.direction() {
            Direction::Horizontal => avail_size.height(),
            Direction::Vertical => avail_size.width(),
        }
    }

    pub fn insets(&self, padding: &geom::Padding) -> (f32, f32) {
        match self.direction() {
            Direction::Horizontal => (padding.left(), padding.right()),
            Direction::Vertical => (padding.bottom(), padding.top()),
        }
    }

    pub fn grid_line_points(
        &self,
        data_num: f64,
        cm: &dyn CoordMap,
        plot_rect: &geom::Rect,
    ) -> (geom::Point, geom::Point) {
        match self {
            Side::Bottom => {
                let x = plot_rect.left() + cm.map_coord_num(data_num);
                let p1 = geom::Point::new(x, plot_rect.bottom());
                let p2 = geom::Point::new(x, plot_rect.top());
                (p1, p2)
            }
            Side::Top => {
                let x = plot_rect.left() + cm.map_coord_num(data_num);
                let p1 = geom::Point::new(x, plot_rect.top());
                let p2 = geom::Point::new(x, plot_rect.bottom());
                (p1, p2)
            }
            Side::Left => {
                let y = plot_rect.bottom() - cm.map_coord_num(data_num);
                let p1 = geom::Point::new(plot_rect.left(), y);
                let p2 = geom::Point::new(plot_rect.right(), y);
                (p1, p2)
            }
            Side::Right => {
                let y = plot_rect.bottom() - cm.map_coord_num(data_num);
                let p1 = geom::Point::new(plot_rect.right(), y);
                let p2 = geom::Point::new(plot_rect.left(), y);
                (p1, p2)
            }
        }
    }

    /// Returns the transform to be applied to the ticks to align them with the axis.
    /// Identity will map ticks horizontally from the top left corner.
    pub fn ticks_marks_transform(&self, rect: &geom::Rect) -> geom::Transform {
        // FIXME: for left axis, and top axis, the ticks are inside out (doesn't matter if symmetrical)
        match self {
            Side::Bottom => geom::Transform::from_translate(rect.left(), rect.bottom()),
            Side::Top => geom::Transform::from_translate(rect.left(), rect.top()),
            Side::Left => {
                geom::Transform::from_translate(rect.left(), rect.bottom()).pre_rotate(-90.0)
            }
            Side::Right => {
                geom::Transform::from_translate(rect.right(), rect.bottom()).pre_rotate(-90.0)
            }
        }
    }
}
