use eidoplot_text as text;
use text::TextLayout;
use text::font;

use crate::drawing::SurfWrapper;
use crate::geom;
use crate::render;
use crate::style;
use crate::style::Color;
use crate::{
    drawing::{
        Categories, Ctx, Error,
        scale::{self, CoordMap},
        ticks,
    },
    ir, missing_params,
    style::theme,
};
use render::Surface;

/// Bounds of an axis
#[derive(Debug, Clone, PartialEq)]
pub enum Bounds {
    /// Numeric bounds, used by both float and integer
    Num(NumBounds),
    /// Category bounds
    Cat(Categories),
}

impl From<NumBounds> for Bounds {
    fn from(value: NumBounds) -> Self {
        Self::Num(value)
    }
}

impl From<Categories> for Bounds {
    fn from(value: Categories) -> Self {
        Self::Cat(value)
    }
}

impl Bounds {
    pub fn unite_with<B>(&mut self, other: &B) -> Result<(), Error>
    where
        B: AsBoundRef,
    {
        let other = other.as_bound_ref();

        match (self, other) {
            (Bounds::Num(a), BoundsRef::Num(b)) => {
                a.unite_with(&b);
                Ok(())
            }
            (Bounds::Cat(a), BoundsRef::Cat(b)) => {
                for s in b.iter() {
                    a.push_if_not_present(s);
                }
                Ok(())
            }
            _ => Err(Error::InconsistentAxisBounds(
                "Cannot unite numerical and categorical axis bounds".into(),
            )),
        }
    }
}

/// Bounds of an axis, borrowing internal its data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsRef<'a> {
    /// Numeric bounds, used by both float and integer
    Num(NumBounds),
    /// Category bounds
    Cat(&'a Categories),
}

impl BoundsRef<'_> {
    pub fn to_bounds(&self) -> Bounds {
        match self {
            &BoundsRef::Num(n) => n.into(),
            &BoundsRef::Cat(c) => c.clone().into(),
        }
    }
}

impl From<NumBounds> for BoundsRef<'_> {
    fn from(value: NumBounds) -> Self {
        Self::Num(value)
    }
}

impl<'a> From<&'a Categories> for BoundsRef<'a> {
    fn from(value: &'a Categories) -> Self {
        Self::Cat(value)
    }
}

impl BoundsRef<'_> {
    pub fn as_num(&self) -> Option<NumBounds> {
        match self {
            &BoundsRef::Num(n) => Some(n),
            _ => None,
        }
    }
}

impl std::cmp::PartialEq<Bounds> for BoundsRef<'_> {
    fn eq(&self, other: &Bounds) -> bool {
        match (self, other) {
            (&BoundsRef::Num(a), &Bounds::Num(b)) => a == b,
            (&BoundsRef::Cat(a), Bounds::Cat(b)) => a == b,
            _ => false,
        }
    }
}

impl std::cmp::PartialEq<BoundsRef<'_>> for Bounds {
    fn eq(&self, other: &BoundsRef) -> bool {
        match (self, other) {
            (&Bounds::Num(a), &BoundsRef::Num(b)) => a == b,
            (Bounds::Cat(a), &BoundsRef::Cat(b)) => a == b,
            _ => false,
        }
    }
}

pub trait AsBoundRef {
    fn as_bound_ref(&self) -> BoundsRef<'_>;
}

impl AsBoundRef for Bounds {
    fn as_bound_ref(&self) -> BoundsRef<'_> {
        match self {
            &Bounds::Num(n) => n.into(),
            &Bounds::Cat(ref c) => c.into(),
        }
    }
}

impl AsBoundRef for BoundsRef<'_> {
    fn as_bound_ref(&self) -> BoundsRef<'_> {
        *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumBounds(f64, f64);

impl NumBounds {
    pub const NAN: Self = Self(f64::NAN, f64::NAN);
}

impl Default for NumBounds {
    fn default() -> Self {
        Self::NAN
    }
}

impl From<f64> for NumBounds {
    fn from(value: f64) -> Self {
        Self(value, value)
    }
}

impl From<(f64, f64)> for NumBounds {
    fn from(value: (f64, f64)) -> Self {
        Self(value.0.min(value.1), value.0.max(value.1))
    }
}

impl NumBounds {
    pub fn start(&self) -> f64 {
        self.0
    }

    pub fn end(&self) -> f64 {
        self.1
    }

    pub fn span(&self) -> f64 {
        self.1 - self.0
    }

    pub fn contains(&self, point: f64) -> bool {
        // TODO: handle very large and very low values
        const EPS: f64 = 1e-10;
        point >= (self.0 - EPS) && point <= (self.1 + EPS)
    }

    pub fn add_sample(&mut self, point: f64) {
        self.0 = self.0.min(point);
        self.1 = self.1.max(point);
    }

    pub fn unite_with(&mut self, bounds: &NumBounds) {
        self.0 = self.0.min(bounds.0);
        self.1 = self.1.max(bounds.1);
    }
}

/// Collection of all axis of a plot
// At the moment, only single bottom and left axis are supported
// TODO: support multiple axes with vec of axis for left and right
#[derive(Debug)]
pub struct Axes {
    pub bottom: Axis,
    pub left: Axis,
}

impl Axes {
    pub fn total_plot_padding(&self) -> geom::Padding {
        geom::Padding::Custom {
            t: 0.0,
            r: 0.0,
            b: self.bottom.size_across(),
            l: self.left.size_across(),
        }
    }
}

#[derive(Debug)]
pub struct Axis {
    /// The location of this axis
    side: Side,
    /// The title of this axis
    title: Option<(TextLayout, theme::Color)>,
    /// The scale of this axis
    scale: AxisScale,
}

impl Axis {
    /// Compute the size of the axis in its orthogonal direction
    /// For horizontal axis, this is the height, for vertical it is the width
    /// The width needs the text layouts
    pub fn size_across(&self) -> f32 {
        let mut size = match &self.scale {
            AxisScale::Num {
                ticks: Some(ticks), ..
            } => ticks.size_across(self.side),
            AxisScale::Cat {
                ticks: Some(ticks), ..
            } => ticks.size_across(self.side),
            _ => 0.0,
        };
        if let Some((title, _)) = self.title.as_ref() {
            // vertical axis rotate the title, therefore we take the height in all cases.
            size += title.height() + missing_params::AXIS_TITLE_MARGIN;
        }
        size
    }

    pub fn set_size_along(&mut self, size: f32) {
        match &mut self.scale {
            AxisScale::Num { cm, .. } => cm.set_plot_size(size),
            AxisScale::Cat { bins, .. } => bins.set_plot_size(size),
        }
    }

    pub fn coord_map(&self) -> &dyn CoordMap {
        match &self.scale {
            AxisScale::Num { cm, .. } => &**cm,
            AxisScale::Cat { bins, .. } => bins,
        }
    }
}

/// Implement the scale for an axis
#[derive(Debug)]
enum AxisScale {
    /// Numerical axis
    Num {
        /// The coordinate mapper
        cm: Box<dyn CoordMap>,
        /// The ticks and labels for the axis
        ticks: Option<NumTicks>,
        /// The minor ticks locations and grid
        minor_ticks: Option<MinorTicks>,
    },
    /// Category axis
    Cat {
        bins: CategoryBins,
        ticks: Option<CategoryTicks>,
    },
}

#[derive(Debug, Clone)]
struct NumTicks {
    /// The color of the ticks labels
    lbl_color: theme::Color,
    /// The list of ticks
    ticks: Vec<NumTick>,
    /// The marker for the ticks
    mark: Option<TickMark>,
    /// Annotation of the axis (e.g. a multiplication factor)
    annot: Option<TextLayout>,
    /// The major grid-lines
    grid: Option<theme::Line>,
}

impl NumTicks {
    fn size_across(&self, side: Side) -> f32 {
        let mut size = 0.0;

        if let Some(mark) = self.mark.as_ref() {
            size += mark.size_out;
        }

        match side {
            Side::Bottom | Side::Top => {
                if let Some(tick) = self.ticks.first() {
                    size += missing_params::TICK_LABEL_MARGIN + tick.lbl.font_size();
                }
            }
            Side::Left | Side::Right => {
                if !self.ticks.is_empty() {
                    size += missing_params::TICK_LABEL_MARGIN;
                }
                let max_w = self
                    .ticks
                    .iter()
                    .map(|t| t.lbl.width())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                size += max_w;
            }
        }
        size
    }
}

#[derive(Debug, Clone)]
struct CategoryTicks {
    lbl_color: theme::Color,
    lbls: Vec<TextLayout>,
    sep: Option<TickMark>,
}

impl CategoryTicks {
    fn size_across(&self, side: Side) -> f32 {
        let mut size = 0.0;

        match side {
            Side::Bottom | Side::Top => {
                if let Some(lbl) = self.lbls.first() {
                    size += missing_params::TICK_LABEL_MARGIN + lbl.font_size();
                }
            }
            Side::Left | Side::Right => {
                if !self.lbls.is_empty() {
                    size += missing_params::TICK_LABEL_MARGIN;
                }
                let max_w = self
                    .lbls
                    .iter()
                    .map(|t| t.width())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                size += max_w;
            }
        }
        size
    }
}

#[derive(Debug, Clone)]
struct NumTick {
    loc: f64,
    lbl: TextLayout,
}

#[derive(Debug, Clone)]
struct TickMark {
    line: theme::Line,
    size_in: f32,
    size_out: f32,
}

#[derive(Debug, Clone)]
struct MinorTicks {
    locs: Vec<f64>,
    mark: Option<TickMark>,
    grid: Option<theme::Line>,
}

#[derive(Debug, Clone)]
struct CategoryBins {
    categories: Categories,
    inset: (f32, f32),
    bin_size: f32,
}

impl CategoryBins {
    fn new(plot_size: f32, inset: (f32, f32), categories: Categories) -> Self {
        // separate the plot_size into one bin per category and place the category in the middle
        let bin_size = (plot_size - inset.0 - inset.1) / categories.len() as f32;
        CategoryBins {
            categories,
            inset,
            bin_size,
        }
    }

    fn len(&self) -> usize {
        self.categories.len()
    }

    /// return the location of a separator before the category at index `cat_idx`
    fn sep_location(&self, cat_idx: usize) -> f32 {
        self.inset.0 + cat_idx as f32 * self.bin_size
    }

    /// return the location of a category at index `cat_idx`
    fn cat_location(&self, cat_idx: usize) -> f32 {
        self.inset.0 + (cat_idx as f32 + 0.5) * self.bin_size
    }
}

impl CoordMap for CategoryBins {
    fn map_coord_cat(&self, cat: &str) -> f32 {
        let cat_idx = self.categories.iter().position(|c| c == cat).unwrap();
        self.cat_location(cat_idx)
    }

    fn axis_bounds(&self) -> BoundsRef<'_> {
        (&self.categories).into()
    }

    fn cat_bin_size(&self) -> f32 {
        self.bin_size
    }

    fn set_plot_size(&mut self, plot_size: f32) {
        self.bin_size = (plot_size - self.inset.0 - self.inset.1) / self.categories.len() as f32;
    }
}

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
    fn title_opts(&self) -> text::layout::Options {
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

    fn title_transform(&self, shift_across: f32, rect: &geom::Rect) -> geom::Transform {
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

    fn ticks_labels_opts(&self) -> text::layout::Options {
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
    fn tick_label_transform(
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

    fn annot_opts(&self) -> text::layout::Options {
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
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
            Side::Right => text::layout::Options {
                hor_align: text::layout::HorAlign::Left,
                ver_align: text::layout::LineVerAlign::Bottom.into(),
                ..Default::default()
            },
        }
    }

    fn annot_transform(&self, shift_across: f32, rect: &geom::Rect) -> geom::Transform {
        match self {
            Side::Bottom => {
                geom::Transform::from_translate(rect.right(), rect.bottom() + shift_across)
            }
            Side::Top => geom::Transform::from_translate(rect.right(), rect.top() - shift_across),
            Side::Left => geom::Transform::from_translate(rect.left() - shift_across, rect.top()),
            Side::Right => geom::Transform::from_translate(rect.right() + shift_across, rect.top()),
        }
    }

    #[allow(dead_code)]
    fn size_along(&self, size: &geom::Size) -> f32 {
        match self.direction() {
            Direction::Horizontal => size.width(),
            Direction::Vertical => size.height(),
        }
    }

    fn size_across(&self, avail_size: &geom::Size) -> f32 {
        match self.direction() {
            Direction::Horizontal => avail_size.height(),
            Direction::Vertical => avail_size.width(),
        }
    }

    fn insets(&self, padding: &geom::Padding) -> (f32, f32) {
        match self.direction() {
            Direction::Horizontal => (padding.left(), padding.right()),
            Direction::Vertical => (padding.bottom(), padding.top()),
        }
    }

    fn grid_line_points(
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
    fn ticks_marks_transform(&self, rect: &geom::Rect) -> geom::Transform {
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

impl<D, T> Ctx<'_, D, T> {
    pub fn setup_axis(
        &self,
        ir: &ir::Axis,
        ab: &Bounds,
        side: Side,
        size_along: f32,
        insets: &geom::Padding,
    ) -> Result<Axis, Error> {
        let insets = side.insets(insets);
        let scale = self.setup_scale(ir, ab, side, size_along, insets)?;

        let title = ir
            .title()
            .map(|title| {
                text::shape_and_layout_str(
                    &title.text,
                    &title.font.font,
                    self.fontdb(),
                    title.font.size,
                    &side.title_opts(),
                )
                .map(|layout| (layout, title.font.color))
            })
            .transpose()?;

        Ok(Axis { side, title, scale })
    }

    fn setup_scale(
        &self,
        ir: &ir::axis::Axis,
        ab: &Bounds,
        side: Side,
        size_along: f32,
        insets: (f32, f32),
    ) -> Result<AxisScale, Error> {
        match ab {
            Bounds::Num(nb) => {
                let cm = scale::map_scale_coord_num(ir.scale(), size_along, &nb, insets);
                let nb = cm.axis_bounds().as_num().unwrap();

                let ticks = ir
                    .ticks()
                    .map(|major_ticks| self.setup_num_ticks(major_ticks, nb, side))
                    .transpose()?;

                let minor_ticks = if let Some(mt) = ir.minor_ticks() {
                    Some(self.setup_minor_ticks(mt, ticks.as_ref(), nb)?)
                } else {
                    None
                };

                Ok(AxisScale::Num {
                    cm,
                    ticks,
                    minor_ticks,
                })
            }
            Bounds::Cat(cats) => {
                let bins = CategoryBins::new(size_along, insets, cats.clone());
                let ticks = ir
                    .ticks()
                    .map(|t| self.setup_cat_ticks(t, cats, side))
                    .transpose()?;
                Ok(AxisScale::Cat { bins, ticks })
            }
        }
    }

    fn setup_num_ticks(
        &self,
        major_ticks: &ir::axis::Ticks,
        nb: NumBounds,
        side: Side,
    ) -> Result<NumTicks, Error> {
        let db: &font::Database = self.fontdb();
        let font = major_ticks.font();
        let grid = major_ticks.grid().cloned();

        let ticks_opts = side.ticks_labels_opts();
        let annot_opts = side.annot_opts();

        let mut major_locs = ticks::locate_num(major_ticks.locator(), nb);
        major_locs.retain(|l| nb.contains(*l));

        let lbl_formatter = ticks::num_label_formatter(major_ticks, nb);
        let mut ticks = Vec::new();
        for loc in major_locs.into_iter() {
            let text = lbl_formatter.format_label(loc.into());
            let lbl = text::shape_and_layout_str(&text, &font.font, db, font.size, &ticks_opts)?;
            ticks.push(NumTick { loc, lbl });
        }

        let annot = lbl_formatter
            .axis_annotation()
            .map(|l| text::shape_and_layout_str(l, &font.font, db, font.size, &annot_opts))
            .transpose()?;

        let mark = Some(TickMark {
            line: major_ticks.color().into(),
            size_in: missing_params::TICK_SIZE,
            size_out: missing_params::TICK_SIZE,
        });

        Ok(NumTicks {
            ticks,
            lbl_color: major_ticks.color(),
            mark,
            annot,
            grid,
        })
    }

    fn setup_minor_ticks(
        &self,
        minor_ticks: &ir::axis::MinorTicks,
        major_ticks: Option<&NumTicks>,
        nb: NumBounds,
    ) -> Result<MinorTicks, Error> {
        let mut locs = ticks::locate_minor(minor_ticks.locator(), nb);
        let major_locs = major_ticks.map(|t| t.ticks.as_slice()).unwrap_or(&[]);

        locs.retain(|l| {
            nb.contains(*l)
                && major_locs
                    .iter()
                    .find(|nt| tick_loc_is_close(nt.loc, *l))
                    .is_none()
        });
        let mark = Some(TickMark {
            line: theme::Col::Foreground.into(),
            size_in: missing_params::MINOR_TICK_SIZE,
            size_out: missing_params::MINOR_TICK_SIZE,
        });

        Ok(MinorTicks {
            locs,
            mark,
            grid: minor_ticks.grid().cloned(),
        })
    }

    fn setup_cat_ticks(
        &self,
        ir: &ir::axis::Ticks,
        cb: &Categories,
        side: Side,
    ) -> Result<CategoryTicks, Error> {
        let db: &font::Database = self.fontdb();
        let font = ir.font();

        let ticks_opts = side.ticks_labels_opts();

        let mut lbls = Vec::with_capacity(cb.len());
        for cat in cb.iter() {
            let layout = text::shape_and_layout_str(cat, &font.font, db, font.size, &ticks_opts)?;
            lbls.push(layout);
        }

        let sep = Some(TickMark {
            line: theme::Col::Foreground.into(),
            size_in: missing_params::TICK_SIZE,
            size_out: missing_params::TICK_SIZE,
        });

        Ok(CategoryTicks {
            lbl_color: ir.color(),
            lbls,
            sep,
        })
    }
}

fn tick_loc_is_close(a: f64, b: f64) -> bool {
    let ratio = a / b;
    ratio.is_finite() && (ratio - 1.0).abs() < 1e-8
}

impl<S: ?Sized> SurfWrapper<'_, S>
where
    S: render::Surface,
{
    pub fn draw_axes_grids<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        axes: &Axes,
        rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        T: style::Theme,
    {
        self.draw_axis_minor_grids(ctx, &axes.bottom, rect)?;
        self.draw_axis_minor_grids(ctx, &axes.left, rect)?;
        self.draw_axis_major_grids(ctx, &axes.bottom, rect)?;
        self.draw_axis_major_grids(ctx, &axes.left, rect)?;
        Ok(())
    }

    fn draw_axis_minor_grids<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        axis: &Axis,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        T: style::Theme,
    {
        let AxisScale::Num {
            cm, minor_ticks, ..
        } = &axis.scale
        else {
            return Ok(());
        };
        if let Some(minor_ticks) = minor_ticks {
            if let Some(grid) = &minor_ticks.grid {
                let mut pathb = geom::PathBuilder::with_capacity(
                    2 * minor_ticks.locs.len(),
                    2 * minor_ticks.locs.len(),
                );
                let stroke = Some(grid.as_stroke(ctx.theme()));
                for t in minor_ticks.locs.iter().copied() {
                    let (p1, p2) = axis.side.grid_line_points(t, &**cm, plot_rect);
                    pathb.move_to(p1.x(), p1.y());
                    pathb.line_to(p2.x(), p2.y());
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke,
                        transform: None,
                    };
                    self.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        Ok(())
    }

    fn draw_axis_major_grids<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        axis: &Axis,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        T: style::Theme,
    {
        let AxisScale::Num { cm, ticks, .. } = &axis.scale else {
            return Ok(());
        };
        if let Some(ticks) = ticks {
            if let Some(grid) = &ticks.grid {
                let mut pathb =
                    geom::PathBuilder::with_capacity(2 * ticks.ticks.len(), 2 * ticks.ticks.len());
                let stroke = Some(grid.as_stroke(ctx.theme()));
                for t in ticks.ticks.iter() {
                    let (p1, p2) = axis.side.grid_line_points(t.loc, &**cm, plot_rect);
                    pathb.move_to(p1.x(), p1.y());
                    pathb.line_to(p2.x(), p2.y());
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke,
                        transform: None,
                    };
                    self.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        Ok(())
    }

    pub fn draw_axes<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        axes: &Axes,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        T: style::Theme,
    {
        self.draw_axis(ctx, &axes.bottom, plot_rect)?;
        self.draw_axis(ctx, &axes.left, plot_rect)?;
        Ok(())
    }

    fn draw_axis<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        axis: &Axis,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        T: style::Theme,
    {
        let mut shift_across = match &axis.scale {
            AxisScale::Num {
                cm,
                ticks,
                minor_ticks,
            } => {
                let mut shift: f32 = 0.0;
                if let Some(minor_ticks) = minor_ticks.as_ref() {
                    shift = shift.max(self.draw_minor_ticks(
                        ctx,
                        &**cm,
                        minor_ticks,
                        axis.side,
                        plot_rect,
                    )?);
                }
                if let Some(ticks) = ticks {
                    shift =
                        shift.max(self.draw_major_ticks(ctx, &**cm, ticks, axis.side, plot_rect)?);
                }
                shift
            }
            AxisScale::Cat { bins, ticks, .. } => {
                if let Some(ticks) = ticks {
                    self.draw_category_ticks(ctx, bins, ticks, axis.side, plot_rect)?
                } else {
                    0.0
                }
            }
        };

        if let Some((layout, color)) = axis.title.as_ref() {
            shift_across += missing_params::AXIS_TITLE_MARGIN;
            let transform = axis.side.title_transform(shift_across, plot_rect);
            let fill = color.resolve(ctx.theme()).into();
            let rtext = render::TextLayout {
                layout,
                fill,
                transform: Some(&transform),
            };
            self.draw_text_layout(&rtext)?;
            // vertical titles are rotated, so it is always the height that is relevant here.
            shift_across += layout.height();
        }
        Ok(shift_across)
    }

    fn draw_major_ticks<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        cm: &dyn CoordMap,
        ticks: &NumTicks,
        side: Side,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        T: style::Theme,
    {
        let shift_across = if let Some(mark) = ticks.mark.as_ref() {
            let transform = side.ticks_marks_transform(plot_rect);
            let ticks = ticks.ticks.iter().map(|t| cm.map_coord_num(t.loc));
            self.draw_ticks_marks(ctx, ticks, mark, &transform)?
        } else {
            0.0
        };

        let color = ticks.lbl_color.resolve(ctx.theme());
        let paint: render::Paint = color.into();

        let shift_across = shift_across + missing_params::TICK_LABEL_MARGIN;
        let mut max_lbl_size: f32 = 0.0;

        for t in ticks.ticks.iter() {
            let lbl_size = geom::Size::new(t.lbl.width(), t.lbl.height());
            max_lbl_size = max_lbl_size.max(side.size_across(&lbl_size));

            let pos_along = cm.map_coord_num(t.loc);
            let transform = side.tick_label_transform(pos_along, shift_across, plot_rect);
            let layout = render::TextLayout {
                layout: &t.lbl,
                fill: paint,
                transform: Some(&transform),
            };
            self.draw_text_layout(&layout)?;
        }
        let shift_across = shift_across + max_lbl_size;
        if let Some(annot) = ticks.annot.as_ref() {
            let transform = side.annot_transform(shift_across, plot_rect);
            let layout = render::TextLayout {
                layout: &annot,
                fill: paint,
                transform: Some(&transform),
            };
            self.draw_text_layout(&layout)?;
        }
        Ok(shift_across)
    }

    fn draw_minor_ticks<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        cm: &dyn CoordMap,
        minor_ticks: &MinorTicks,
        side: Side,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        T: style::Theme,
    {
        let Some(mark) = minor_ticks.mark.as_ref() else {
            return Ok(0.0);
        };
        let transform = side.ticks_marks_transform(plot_rect);
        let ticks = minor_ticks
            .locs
            .iter()
            .copied()
            .map(|t| cm.map_coord_num(t));
        self.draw_ticks_marks(ctx, ticks, mark, &transform)
    }

    fn draw_category_ticks<D, T>(
        &mut self,
        ctx: &Ctx<D, T>,
        bins: &CategoryBins,
        ticks: &CategoryTicks,
        side: Side,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        T: style::Theme,
    {
        if let Some(sep) = ticks.sep.as_ref() {
            let locs = (0..bins.len() + 1).map(|i| bins.sep_location(i));
            let transform = side.ticks_marks_transform(plot_rect);
            self.draw_ticks_marks(ctx, locs, sep, &transform)?;
        }
        // tick marks are separators, so not counted in shift_across, because not supposed to overlap
        let shift_across = missing_params::TICK_LABEL_MARGIN;

        let color = ticks.lbl_color.resolve(ctx.theme());
        let fill: render::Paint = color.into();

        let mut max_lbl_size: f32 = 0.0;

        for (i, lbl) in ticks.lbls.iter().enumerate() {
            let txt_size = geom::Size::new(lbl.width(), lbl.height());
            max_lbl_size = max_lbl_size.max(side.size_across(&txt_size));

            let pos_along = bins.cat_location(i);
            let transform = side.tick_label_transform(pos_along, shift_across, plot_rect);
            let layout = render::TextLayout {
                layout: lbl,
                fill,
                transform: Some(&transform),
            };
            self.draw_text_layout(&layout)?;
        }

        Ok(shift_across + max_lbl_size)
    }

    // return shift across axis (distance to get away from axis to avoid collision)
    fn draw_ticks_marks<D, T, I>(
        &mut self,
        ctx: &Ctx<D, T>,
        ticks: I,
        mark: &TickMark,
        transform: &geom::Transform,
    ) -> Result<f32, Error>
    where
        T: style::Theme,
        I: Iterator<Item = f32>,
    {
        let mut pb = geom::PathBuilder::new();
        for t in ticks {
            pb.move_to(t, -mark.size_in);
            pb.line_to(t, mark.size_out);
        }
        let path = pb.finish().expect("Should be a valid path");
        let rpath = render::Path {
            path: &path,
            fill: None,
            stroke: Some(mark.line.as_stroke(ctx.theme())),
            transform: Some(transform),
        };
        self.draw_path(&rpath)?;
        Ok(mark.size_out)
    }
}
