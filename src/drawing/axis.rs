use std::sync::Arc;

mod bounds;
mod side;

pub use bounds::{AsBoundRef, Bounds, BoundsRef, NumBounds, TimeBounds};
pub use side::Side;

use crate::drawing::scale::{self, CoordMap};
use crate::drawing::{Categories, Ctx, Error, Text, ticks};
use crate::style::{Theme, theme};
use crate::text::{self, font};
use crate::{data, geom, ir, missing_params, render};

#[derive(Debug, Clone)]
pub struct Axis {
    id: Option<String>,
    title_text: Option<String>,
    side: Side,
    draw_opts: DrawOpts,
    scale: Arc<AxisScale>,
}

impl Axis {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn title_text(&self) -> Option<&str> {
        self.title_text.as_deref()
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn scale(&self) -> &Arc<AxisScale> {
        &self.scale
    }

    pub fn size_across(&self) -> f32 {
        let mark_size = self.draw_opts.marks.as_ref().map_or(0.0, |m| m.size_out);
        let with_labels = self.draw_opts.ticks_labels;
        let mut size = match self.scale.as_ref() {
            AxisScale::Num {
                ticks: Some(ticks), ..
            } => ticks.size_across(self.side, mark_size, with_labels),
            AxisScale::Cat {
                ticks: Some(ticks), ..
            } => ticks.size_across(self.side, mark_size, with_labels),
            _ => 0.0,
        };
        if let Some(title) = self.draw_opts.title.as_ref() {
            // vertical axis rotate the title, therefore we take the height in all cases.
            size += title.bbox.height() + missing_params::AXIS_TITLE_MARGIN;
        }
        size
    }

    pub fn coord_map(&self) -> &dyn CoordMap {
        match self.scale.as_ref() {
            AxisScale::Num { cm, .. } => &**cm,
            AxisScale::Cat { bins, .. } => bins,
        }
    }
}

/// Implement the scale for an axis
#[derive(Debug)]
pub enum AxisScale {
    /// Numerical axis
    Num {
        /// The coordinate mapper
        cm: Box<dyn CoordMap>,
        /// The ticks and labels for the axis
        ticks: Option<NumTicks>,
        /// The minor ticks locations
        minor_ticks: Option<MinorTicks>,
    },
    /// Category axis
    Cat {
        bins: CategoryBins,
        ticks: Option<CategoryTicks>,
    },
}

#[derive(Debug, Clone)]
pub struct NumTicks {
    /// The list of ticks
    ticks: Vec<NumTick>,
    /// Annotation of the axis (e.g. a multiplication factor)
    annot: Option<Text>,
}

impl NumTicks {
    fn size_across(&self, side: Side, mark_size: f32, with_labels: bool) -> f32 {
        // mark_size is only accounted for when there are labels
        // this allows to merge ticks of subplots with shared scales and zero inter-space
        if !with_labels {
            return 0.0;
        }

        let mut size = mark_size;

        if !self.ticks.is_empty() {
            size += missing_params::TICK_LABEL_MARGIN;
        }

        match side {
            Side::Bottom | Side::Top => {
                let max_h = self
                    .ticks
                    .iter()
                    .map(|t| t.lbl.bbox.height())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                size += max_h;
            }
            Side::Left | Side::Right => {
                let max_w = self
                    .ticks
                    .iter()
                    .map(|t| t.lbl.bbox.width())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                size += max_w;
            }
        }
        size
    }
}

/// A numeric tick location and its label
#[derive(Debug, Clone)]
struct NumTick {
    loc: f64,
    lbl: Text,
}

#[derive(Debug, Clone)]
struct TickMark {
    line: theme::Line,
    size_in: f32,
    size_out: f32,
}

#[derive(Debug, Clone)]
pub struct MinorTicks {
    locs: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct CategoryBins {
    categories: Categories,
    inset: (f32, f32),
    bin_size: f32,
}

impl CategoryBins {
    fn new(plot_size: f32, inset: (f32, f32), categories: Categories) -> Self {
        // separate the plot_size into one bin per category and place the category in the middle
        let bin_size = Self::calc_bin_size(plot_size, inset, categories.len());
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

    fn calc_bin_size(plot_size: f32, inset: (f32, f32), n_cats: usize) -> f32 {
        (plot_size - inset.0 - inset.1) / n_cats as f32
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
}

#[derive(Debug, Clone)]
pub struct CategoryTicks {
    font_size: f32,
    lbls: Vec<Text>,
    sep: Option<TickMark>,
}

impl CategoryTicks {
    fn size_across(&self, side: Side, mark_size: f32, with_labels: bool) -> f32 {
        // Marks are separators rather than ticks, they don't shift the labels.
        // As such, they are only counted if labels are not there.

        if !with_labels {
            return mark_size;
        }

        let mut size = 0.0;

        match side {
            Side::Bottom | Side::Top => {
                if !self.lbls.is_empty() {
                    size += missing_params::TICK_LABEL_MARGIN + self.font_size;
                }
            }
            Side::Left | Side::Right => {
                if !self.lbls.is_empty() {
                    size += missing_params::TICK_LABEL_MARGIN;
                }
                let max_w = self
                    .lbls
                    .iter()
                    .map(|t| t.bbox.width())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                size += max_w;
            }
        }
        size
    }
}

/// Axis drawing options
/// Especially important in the context of subplots and shared axes
/// The location of ticks and their labels is determined by the shared scale
#[derive(Debug, Clone)]
struct DrawOpts {
    title: Option<Text>,
    spine: Option<ir::plot::Border>,
    marks: Option<TickMark>,
    minor_marks: Option<TickMark>,
    ticks_labels: bool,
    grid: Option<theme::Line>,
    minor_grid: Option<theme::Line>,
}

impl<D> Ctx<'_, D>
where
    D: data::Source,
{
    /// Estimate the height taken by a horizontal axis.
    /// It includes ticks marks, ticks labels and axis title.
    /// This is the height without any additional margin
    pub fn estimate_x_axes_height(&self, x_axes: &[ir::Axis], side: ir::axis::Side) -> f32 {
        let mut height = 0.0;
        for (idx, axis) in x_axes.iter().filter(|a| a.side() == side).enumerate() {
            if idx != 0 {
                height += missing_params::AXIS_MARGIN + missing_params::AXIS_SPINE_WIDTH;
            }
            if let Some(ticks) = axis.ticks() {
                if axis.has_tick_labels() {
                    // ticks is only accounted for when there are labels
                    // this allows to merge ticks of subplots with shared scales and zero inter-space
                    if idx != 0 {
                        height += missing_params::TICK_SIZE;
                    }
                    height += missing_params::TICK_SIZE;
                    height += missing_params::TICK_LABEL_MARGIN + ticks.font().size;
                }
            }
            if let Some(title) = axis.title() {
                height += missing_params::AXIS_TITLE_MARGIN + title.props().font_size();
            }
        }
        height
    }

    pub fn setup_axis(
        &self,
        ir_axis: &ir::Axis,
        bounds: &Bounds,
        side: Side,
        size_along: f32,
        insets: &geom::Padding,
        shared_scale: Option<Arc<AxisScale>>,
        spine: Option<ir::plot::Border>,
    ) -> Result<Axis, Error> {
        let id = ir_axis.id().map(|s| s.to_string());
        let title_text = ir_axis.title().map(|t| t.text().to_string());

        let uses_shared = shared_scale.is_some();
        let draw_opts = self.setup_axis_draw_opts(ir_axis, side, uses_shared, spine)?;

        let scale = if let Some(scale) = shared_scale {
            scale
        } else {
            let insets = side.insets(insets);
            Arc::new(self.setup_axis_scale(ir_axis, bounds, side, size_along, insets)?)
        };

        Ok(Axis {
            id,
            title_text,
            side,
            draw_opts,
            scale,
        })
    }

    fn setup_axis_scale(
        &self,
        ir_axis: &ir::Axis,
        bounds: &Bounds,
        side: Side,
        size_along: f32,
        insets: (f32, f32),
    ) -> Result<AxisScale, Error> {
        match bounds {
            Bounds::Num(nb) => {
                let cm = scale::map_scale_coord_num(ir_axis.scale(), size_along, &nb, insets);
                let nb = cm.axis_bounds().as_num().unwrap();

                let ticks = ir_axis
                    .ticks()
                    .map(|major_ticks| self.setup_num_ticks(major_ticks, nb, ir_axis.scale(), side))
                    .transpose()?;

                let minor_ticks = if let Some(mt) = ir_axis.minor_ticks() {
                    Some(self.setup_minor_ticks(mt, ticks.as_ref(), ir_axis.scale(), nb)?)
                } else {
                    None
                };

                Ok(AxisScale::Num {
                    cm,
                    ticks,
                    minor_ticks,
                })
            }
            Bounds::Time(tb) => {
                let nb: NumBounds = (*tb).into();
                let cm = scale::map_scale_coord_num(ir_axis.scale(), size_along, &nb, insets);
                let nb = cm.axis_bounds().as_num().unwrap();
                let tb: TimeBounds = nb.into();

                let ticks = ir_axis
                    .ticks()
                    .map(|major_ticks| {
                        self.setup_time_ticks(major_ticks, tb, ir_axis.scale(), side)
                    })
                    .transpose()?;

                if ir_axis.minor_ticks().is_some() {
                    return Err(Error::InconsistentIr(
                        "Minor ticks not supported for time axis".into(),
                    ));
                }

                Ok(AxisScale::Num {
                    cm,
                    ticks,
                    minor_ticks: None,
                })
            }
            Bounds::Cat(cats) => {
                let bins = CategoryBins::new(size_along, insets, cats.clone());
                let ticks = ir_axis
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
        scale: &ir::axis::Scale,
        side: Side,
    ) -> Result<NumTicks, Error> {
        let db: &font::Database = self.fontdb();
        let font = major_ticks.font();

        let ticks_align = side.ticks_labels_align();
        let annot_align = side.annot_align();

        let mut major_locs = ticks::locate_num(major_ticks.locator(), nb, scale)?;
        major_locs.retain(|l| nb.contains(*l));

        let lbl_formatter = ticks::num_label_formatter(major_ticks, nb, scale);
        let mut ticks = Vec::new();
        for loc in major_locs.into_iter() {
            let text = lbl_formatter.format_label(loc.into());
            let lbl = text::LineText::new(text, ticks_align, font.size, font.font.clone(), db)?;
            let lbl = Text::from_line_text(&lbl, db, major_ticks.color())?;
            ticks.push(NumTick { loc, lbl });
        }

        let annot = lbl_formatter
            .axis_annotation()
            .map(|l| {
                text::LineText::new(l.to_string(), annot_align, font.size, font.font.clone(), db)
            })
            .transpose()?
            .map(|lbl| Text::from_line_text(&lbl, db, major_ticks.color()))
            .transpose()?;

        Ok(NumTicks { ticks, annot })
    }

    fn setup_minor_ticks(
        &self,
        minor_ticks: &ir::axis::MinorTicks,
        major_ticks: Option<&NumTicks>,
        scale: &ir::axis::Scale,
        nb: NumBounds,
    ) -> Result<MinorTicks, Error> {
        let mut locs = ticks::locate_minor(minor_ticks.locator(), nb, scale)?;
        let major_locs = major_ticks.map(|t| t.ticks.as_slice()).unwrap_or(&[]);

        locs.retain(|l| {
            nb.contains(*l)
                && major_locs
                    .iter()
                    .find(|nt| tick_loc_is_close(nt.loc, *l))
                    .is_none()
        });

        Ok(MinorTicks { locs })
    }

    fn setup_time_ticks(
        &self,
        major_ticks: &ir::axis::Ticks,
        tb: TimeBounds,
        scale: &ir::axis::Scale,
        side: Side,
    ) -> Result<NumTicks, Error> {
        let db: &font::Database = self.fontdb();
        let font = major_ticks.font();

        let ticks_align = side.ticks_labels_align();
        let annot_align = side.annot_align();

        if matches!(scale, ir::axis::Scale::Log(_)) {
            return Err(Error::InconsistentIr(
                "Log scale not supported for time axis".into(),
            ));
        }

        let mut major_locs = ticks::locate_datetime(major_ticks.locator(), tb)?;
        major_locs.retain(|l| tb.contains(*l));

        let lbl_formatter = ticks::datetime_label_formatter(major_ticks, tb, scale)?;
        let mut ticks = Vec::new();
        for loc in major_locs.into_iter() {
            let text = lbl_formatter.format_label(loc.into());
            let lbl = text::LineText::new(text, ticks_align, font.size, font.font.clone(), db)?;
            let lbl = Text::from_line_text(&lbl, db, major_ticks.color())?;
            ticks.push(NumTick {
                loc: loc.timestamp(),
                lbl,
            });
        }

        let annot = lbl_formatter
            .axis_annotation()
            .map(|l| {
                text::LineText::new(l.to_string(), annot_align, font.size, font.font.clone(), db)
            })
            .transpose()?
            .map(|lbl| Text::from_line_text(&lbl, db, major_ticks.color()))
            .transpose()?;

        Ok(NumTicks { ticks, annot })
    }

    fn setup_cat_ticks(
        &self,
        ir: &ir::axis::Ticks,
        cb: &Categories,
        side: Side,
    ) -> Result<CategoryTicks, Error> {
        let db: &font::Database = self.fontdb();
        let font = ir.font();

        let ticks_align = side.ticks_labels_align();

        let mut lbls = Vec::with_capacity(cb.len());
        for cat in cb.iter() {
            let lbl = text::LineText::new(
                cat.to_string(),
                ticks_align,
                font.size,
                font.font.clone(),
                db,
            )?;
            let lbl = Text::from_line_text(&lbl, db, ir.color())?;
            lbls.push(lbl);
        }

        let sep = Some(TickMark {
            line: theme::Col::Foreground.into(),
            size_in: missing_params::TICK_SIZE,
            size_out: missing_params::TICK_SIZE,
        });

        Ok(CategoryTicks {
            font_size: font.size,
            lbls,
            sep,
        })
    }

    fn setup_axis_draw_opts(
        &self,
        ir_axis: &ir::Axis,
        side: Side,
        uses_shared: bool,
        spine: Option<ir::plot::Border>,
    ) -> Result<DrawOpts, Error> {
        let title = ir_axis
            .title()
            .map(|title| title.to_rich_text(side.title_layout(), &self.fontdb))
            .transpose()?
            .map(|rich| Text::from_rich_text(&rich, &self.fontdb))
            .transpose()?;

        let ticks_labels = !uses_shared;
        let marks = ir_axis.ticks().map(|ticks| TickMark {
            line: ticks.color().into(),
            size_in: missing_params::TICK_SIZE,
            size_out: missing_params::TICK_SIZE,
        });
        let minor_marks = ir_axis.minor_ticks().map(|ticks| TickMark {
            line: theme::Line::from(ticks.color())
                .with_width(missing_params::MINOR_TICK_LINE_WIDTH),
            size_in: missing_params::MINOR_TICK_SIZE,
            size_out: missing_params::MINOR_TICK_SIZE,
        });
        let grid = ir_axis.grid().map(|grid| grid.0.clone());
        let minor_grid = ir_axis.minor_grid().map(|grid| grid.0.clone());

        Ok(DrawOpts {
            title,
            spine,
            ticks_labels,
            marks,
            minor_marks,
            grid,
            minor_grid,
        })
    }
}

fn tick_loc_is_close(a: f64, b: f64) -> bool {
    let ratio = a / b;
    ratio.is_finite() && (ratio - 1.0).abs() < 1e-8
}

impl Axis {
    pub fn draw_minor_grids<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        S: render::Surface,
    {
        let AxisScale::Num {
            cm, minor_ticks, ..
        } = self.scale.as_ref()
        else {
            return Ok(());
        };

        if let Some(minor_ticks) = minor_ticks {
            if let Some(grid) = &self.draw_opts.minor_grid {
                let mut pathb = geom::PathBuilder::with_capacity(
                    2 * minor_ticks.locs.len(),
                    2 * minor_ticks.locs.len(),
                );
                let stroke = Some(grid.as_stroke(theme));
                for t in minor_ticks.locs.iter().copied() {
                    let (p1, p2) = self.side.grid_line_points(t, &**cm, plot_rect);
                    pathb.move_to(p1.x, p1.y);
                    pathb.line_to(p2.x, p2.y);
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke,
                        transform: None,
                    };
                    surface.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        Ok(())
    }

    pub fn draw_major_grids<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        plot_rect: &geom::Rect,
    ) -> Result<(), Error>
    where
        S: render::Surface,
    {
        let AxisScale::Num { cm, ticks, .. } = self.scale.as_ref() else {
            return Ok(());
        };
        if let Some(ticks) = ticks {
            if let Some(grid) = &self.draw_opts.grid {
                let mut pathb =
                    geom::PathBuilder::with_capacity(2 * ticks.ticks.len(), 2 * ticks.ticks.len());
                let stroke = Some(grid.as_stroke(theme));
                for t in ticks.ticks.iter() {
                    let (p1, p2) = self.side.grid_line_points(t.loc, &**cm, plot_rect);
                    pathb.move_to(p1.x, p1.y);
                    pathb.line_to(p2.x, p2.y);
                    let path = pathb.finish().expect("Should be a valid path");
                    let rpath = render::Path {
                        path: &path,
                        fill: None,
                        stroke,
                        transform: None,
                    };
                    surface.draw_path(&rpath)?;
                    pathb = path.clear();
                }
            }
        }
        Ok(())
    }

    pub fn draw<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        S: render::Surface,
    {
        if let Some(spine) = self.draw_opts.spine.as_ref() {
            self.draw_spine(surface, theme, plot_rect, spine)?;
        }

        let mut shift_across = match self.scale.as_ref() {
            AxisScale::Num {
                cm,
                ticks,
                minor_ticks,
            } => {
                let mut shift: f32 = 0.0;
                if let Some(minor_ticks) = minor_ticks.as_ref() {
                    shift = shift.max(self.draw_minor_ticks(
                        surface,
                        theme,
                        &**cm,
                        minor_ticks,
                        plot_rect,
                    )?);
                }
                if let Some(ticks) = ticks {
                    shift =
                        shift.max(self.draw_major_ticks(surface, theme, &**cm, ticks, plot_rect)?);
                }
                shift
            }
            AxisScale::Cat { bins, ticks, .. } => {
                if let Some(ticks) = ticks {
                    self.draw_category_ticks(surface, theme, bins, ticks, plot_rect)?
                } else {
                    0.0
                }
            }
        };

        if let Some(title) = self.draw_opts.title.as_ref() {
            shift_across += missing_params::AXIS_TITLE_MARGIN;
            let transform = self.side.title_transform(shift_across, plot_rect);
            title.draw(surface, theme, Some(&transform))?;
            // vertical titles are rotated, so it is always the height that is relevant here.
            shift_across += title.bbox.height();
        }
        Ok(shift_across)
    }

    fn draw_major_ticks<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        cm: &dyn CoordMap,
        ticks: &NumTicks,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        S: render::Surface,
    {
        let mut shift_across = 0.0;

        if let Some(mark) = self.draw_opts.marks.as_ref() {
            let transform = self.side.ticks_marks_transform(plot_rect);
            let ticks = ticks.ticks.iter().map(|t| cm.map_coord_num(t.loc));
            shift_across += self.draw_ticks_marks(surface, theme, ticks, mark, &transform)?
        }

        if !self.draw_opts.ticks_labels {
            return Ok(shift_across);
        }

        shift_across += missing_params::TICK_LABEL_MARGIN;
        let mut max_lbl_size: f32 = 0.0;

        for t in ticks.ticks.iter() {
            let lbl_size = geom::Size::new(t.lbl.bbox.width(), t.lbl.bbox.height());
            max_lbl_size = max_lbl_size.max(self.side.size_across(&lbl_size));

            let pos_along = cm.map_coord_num(t.loc);
            let transform = self
                .side
                .tick_label_transform(pos_along, shift_across, plot_rect);
            t.lbl.draw(surface, theme, Some(&transform))?;
        }

        shift_across += max_lbl_size;

        if let Some(annot) = ticks.annot.as_ref() {
            let transform = self.side.annot_transform(shift_across, plot_rect);
            annot.draw(surface, theme, Some(&transform))?;
        }
        Ok(shift_across)
    }

    fn draw_spine<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        plot_rect: &geom::Rect,
        spine: &ir::plot::Border,
    ) -> Result<(), Error>
    where
        S: render::Surface,
    {
        let stroke = spine.line().as_stroke(theme);
        let path = self.side.spine_path(plot_rect, spine);
        let rpath = render::Path {
            path: &path,
            fill: None,
            stroke: Some(stroke),
            transform: None,
        };
        surface.draw_path(&rpath)?;
        Ok(())
    }

    fn draw_minor_ticks<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        cm: &dyn CoordMap,
        minor_ticks: &MinorTicks,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        S: render::Surface,
    {
        let Some(mark) = self.draw_opts.minor_marks.as_ref() else {
            return Ok(0.0);
        };
        let transform = self.side.ticks_marks_transform(plot_rect);
        let ticks = minor_ticks
            .locs
            .iter()
            .copied()
            .map(|t| cm.map_coord_num(t));
        self.draw_ticks_marks(surface, theme, ticks, mark, &transform)
    }

    fn draw_category_ticks<S>(
        &self,
        surface: &mut S,
        theme: &Theme,
        bins: &CategoryBins,
        ticks: &CategoryTicks,
        plot_rect: &geom::Rect,
    ) -> Result<f32, Error>
    where
        S: render::Surface,
    {
        if let Some(sep) = ticks.sep.as_ref() {
            let locs = (0..bins.len() + 1).map(|i| bins.sep_location(i));
            let transform = self.side.ticks_marks_transform(plot_rect);
            self.draw_ticks_marks(surface, theme, locs, sep, &transform)?;
        }
        // tick marks are separators, so not counted in shift_across, because not supposed to overlap
        let shift_across = missing_params::TICK_LABEL_MARGIN;

        let mut max_lbl_size: f32 = 0.0;

        for (i, lbl) in ticks.lbls.iter().enumerate() {
            let txt_size = geom::Size::new(lbl.bbox.width(), lbl.bbox.height());
            max_lbl_size = max_lbl_size.max(self.side.size_across(&txt_size));

            let pos_along = bins.cat_location(i);
            let transform = self
                .side
                .tick_label_transform(pos_along, shift_across, plot_rect);
            lbl.draw(surface, theme, Some(&transform))?;
        }

        Ok(shift_across + max_lbl_size)
    }

    // return shift across axis (distance to get away from axis to avoid collision)
    fn draw_ticks_marks<S, I>(
        &self,
        surface: &mut S,
        theme: &Theme,
        ticks: I,
        mark: &TickMark,
        transform: &geom::Transform,
    ) -> Result<f32, Error>
    where
        S: render::Surface,
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
            stroke: Some(mark.line.as_stroke(theme)),
            transform: Some(transform),
        };
        surface.draw_path(&rpath)?;
        Ok(mark.size_out)
    }
}
