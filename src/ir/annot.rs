//! Annotations to place on the plot area.
use crate::ir::axis;
use crate::style::{self, theme};
use crate::text::Font;

/// An arbitrary graphical annotation placed on the plot area.
/// The placement is made according to the data coordinates.
/// By default, lines are plotted under the series, and other annotations are plotted above the series.
/// This can be changed using [`with_zpos()`].
#[derive(Debug, Clone)]
pub enum Annotation {
    /// A line plotted on the plot area.
    Line(Line),
    /// An arrow plotted on the plot area.
    Arrow(Arrow),
    /// A marker plotted on the plot area.
    Marker(Marker),
    /// A label plotted on the plot area.
    Label(Label),
}

impl From<Line> for Annotation {
    fn from(line: Line) -> Self {
        Annotation::Line(line)
    }
}

impl From<Arrow> for Annotation {
    fn from(arrow: Arrow) -> Self {
        Annotation::Arrow(arrow)
    }
}

impl From<Marker> for Annotation {
    fn from(marker: Marker) -> Self {
        Annotation::Marker(marker)
    }
}

impl From<Label> for Annotation {
    fn from(label: Label) -> Self {
        Annotation::Label(label)
    }
}

impl Annotation {
    pub(crate) fn pos_mut(&mut self) -> &mut Pos {
        match self {
            Annotation::Line(line) => &mut line.pos,
            Annotation::Arrow(arrow) => &mut arrow.pos,
            Annotation::Marker(marker) => &mut marker.pos,
            Annotation::Label(label) => &mut label.pos,
        }
    }

    /// Set the X-axis to use for this label.
    /// Only useful if multiple X-axes are used.
    /// By default, the first X-axis is used.
    pub fn with_x_axis(mut self, x_axis: axis::Ref) -> Self {
        self.pos_mut().x_axis = x_axis;
        self
    }

    /// Set the Y-axis to use for this label.
    /// Only useful if multiple Y-axes are used.
    /// By default, the first Y-axis is used.
    pub fn with_y_axis(mut self, y_axis: axis::Ref) -> Self {
        self.pos_mut().y_axis = y_axis;
        self
    }

    /// Set the z-position of this annotation in relation to the series.
    pub fn with_zpos(mut self, zpos: ZPos) -> Self {
        self.pos_mut().zpos = zpos;
        self
    }
}

/// Positioning information for annotations placed on the plot area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZPos {
    /// Annotation displayed below the series
    BelowSeries,
    /// Annotation displayed above the series
    AboveSeries,
}

#[derive(Debug, Clone)]
pub(crate) struct Pos {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) x_axis: axis::Ref,
    pub(crate) y_axis: axis::Ref,
    pub(crate) zpos: ZPos,
}

/// A line plotted on the plot area.
#[derive(Debug, Clone)]
pub struct Line {
    pub(crate) direction: Direction,
    pub(crate) line: theme::Line,

    pub(crate) pos: Pos,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Horizontal,
    Vertical,
    Slope(f32),
    SecondPoint(f64, f64),
}

impl Line {
    /// Plot a vertical line passing by x
    pub fn vertical(x: f64) -> Self {
        Line {
            direction: Direction::Vertical,
            line: theme::Col::Foreground.into(),
            pos: Pos {
                x,
                y: 0.0,
                x_axis: Default::default(),
                y_axis: Default::default(),
                zpos: ZPos::BelowSeries,
            },
        }
    }

    /// Plot a horizontal line passing by y
    pub fn horizontal(y: f64) -> Self {
        Line {
            direction: Direction::Horizontal,
            line: theme::Col::Foreground.into(),
            pos: Pos {
                x: 0.0,
                y,
                x_axis: Default::default(),
                y_axis: Default::default(),
                zpos: ZPos::BelowSeries,
            },
        }
    }

    /// Plot a line passing by x and y with the given slope.
    /// This is only meaningful on linear scales, and will raise an error
    /// if either X or Y axes are logarithmic.
    pub fn slope(x: f64, y: f64, slope: f32) -> Self {
        Line {
            direction: Direction::Slope(slope),
            line: theme::Col::Foreground.into(),
            pos: Pos {
                x,
                y,
                x_axis: Default::default(),
                y_axis: Default::default(),
                zpos: ZPos::BelowSeries,
            },
        }
    }

    /// Plot a line passing by (x1, y1) and (x2, y2).
    pub fn two_points(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Line {
            direction: Direction::SecondPoint(x2, y2),
            line: theme::Col::Foreground.into(),
            pos: Pos {
                x: x1,
                y: y1,
                x_axis: Default::default(),
                y_axis: Default::default(),
                zpos: ZPos::BelowSeries,
            },
        }
    }

    /// Set the line to be displayed.
    /// By default, the line is a solid line of the foreground theme color.
    pub fn with_line(self, line: theme::Line) -> Self {
        Self { line, ..self }
    }

    /// Set the pattern of the line
    pub fn with_pattern(self, pattern: style::LinePattern) -> Self {
        Self {
            line: self.line.with_pattern(pattern),
            ..self
        }
    }
}

/// An arrow plotted on the plot area
#[derive(Debug, Clone)]
pub struct Arrow {
    pub(crate) dx: f32,
    pub(crate) dy: f32,
    pub(crate) head_size: f32,
    pub(crate) line: theme::Line,

    pub(crate) pos: Pos,
}

impl Arrow {
    /// Create a new arrow pointing at (x, y) in data coordinates,
    /// with the given delta vector in figure units.
    pub fn new(x: f64, y: f64, dx: f32, dy: f32) -> Self {
        Arrow {
            dx,
            dy,
            head_size: 10.0,
            line: theme::Col::Foreground.into(),
            pos: Pos {
                x,
                y,
                x_axis: Default::default(),
                y_axis: Default::default(),
                zpos: ZPos::AboveSeries,
            },
        }
    }

    /// Set the line style of the arrow.
    /// By default the foreground theme color is used with a solid line of width 1.0.
    pub fn with_line(self, line: theme::Line) -> Self {
        Self { line, ..self }
    }

    /// Set the head size of the arrow in figure units. By default 5.0.
    pub fn with_head_size(self, head_size: f32) -> Self {
        Self { head_size, ..self }
    }
}

/// An arbitrary marker to place on the plot area
#[derive(Debug, Clone)]
pub struct Marker {
    pub(crate) marker: theme::Marker,
    pub(crate) pos: Pos,
}

/// An anchor point for [`PlotLabel`].
/// It defines which point of the label is positioned at the given data coordinates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Anchor {
    /// Anchor at the center of the label
    Center,
    #[default]
    /// Anchor at the top-left of the label
    TopLeft,
    /// Anchor at the top-right of the label
    TopRight,
    /// Anchor at the bottom-right of the label
    BottomRight,
    /// Anchor at the bottom-left of the label
    BottomLeft,
    /// Anchor at the top-center of the label
    TopCenter,
    /// Anchor at the center-right of the label
    CenterRight,
    /// Anchor at the bottom-center of the label
    BottomCenter,
    /// Anchor at the center-left of the label
    CenterLeft,
}

/// An arbitrary label to place on the plot area
#[derive(Debug, Clone)]
pub struct Label {
    pub(crate) text: String,
    pub(crate) font_size: f32,
    pub(crate) font: Font,
    pub(crate) color: theme::Color,
    pub(crate) anchor: Anchor,
    pub(crate) frame: (Option<theme::Fill>, Option<theme::Line>),
    pub(crate) angle: f32,

    pub(crate) pos: Pos,
}

impl Label {
    /// Create a new label with the given text at data coordinates (x, y)
    pub fn new(text: String, x: f64, y: f64) -> Self {
        Label {
            text,
            font_size: 12.0,
            font: Font::default(),
            color: theme::Col::Foreground.into(),
            anchor: Anchor::default(),
            frame: (None, None),
            angle: 0.0,
            pos: Pos {
                x,
                y,
                x_axis: Default::default(),
                y_axis: Default::default(),
                zpos: ZPos::AboveSeries,
            },
        }
    }

    /// Set the font size of the label
    pub fn with_font_size(self, font_size: f32) -> Self {
        Self { font_size, ..self }
    }

    /// Set the font of the label
    pub fn with_font(self, font: Font) -> Self {
        Self { font, ..self }
    }

    /// Set the color of the label.
    /// By default, the foreground theme color is used.
    pub fn with_color(self, color: theme::Color) -> Self {
        Self { color, ..self }
    }

    /// Set the anchor point of the label.
    /// By default, the top-left corner is used.
    pub fn with_anchor(self, anchor: Anchor) -> Self {
        Self { anchor, ..self }
    }

    /// Set the frame border and fill of the label.
    /// By default, there is no frame.
    pub fn with_frame(self, fill: Option<theme::Fill>, stroke: Option<theme::Line>) -> Self {
        Self {
            frame: (fill, stroke),
            ..self
        }
    }

    /// Set the rotation angle of the label in degrees in counter-clockwise direction.
    /// The label is rotated around its anchor point.
    /// By default, the angle is 0.0 (horizontal).
    pub fn with_angle(self, angle: f32) -> Self {
        Self { angle, ..self }
    }
}
