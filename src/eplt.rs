//! EPLT DSL parser
//!
//! EPLT is a domain-specific language for defining plots and figures.
//! An `*.eplt*` file contains one or more [`ir::Figure`] definitions.
use std::fmt;
#[cfg(feature = "dsl-diag")]
use std::path;

#[cfg(feature = "dsl-diag")]
pub use dsl::{Diagnostic, Source};

use crate::dsl::{self, ast};
use crate::text::{self, ParseRichTextError, ParsedRichText};
use crate::{ir, style};

/// Errors that can occur during EPLT parsing
#[derive(Debug, Clone)]
pub enum Error {
    /// DSL parsing error
    Dsl(dsl::Error),
    /// Rich text parsing error with offset
    ParseRichText(usize, ParseRichTextError),
    /// General parse error
    Parse {
        /// Span of the error
        span: dsl::Span,
        /// Reason for the error
        reason: String,
        /// Optional help message
        help: Option<String>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Dsl(err) => err.fmt(f),
            Error::ParseRichText(_, err) => err.fmt(f),
            Error::Parse { reason, help, .. } => {
                write!(f, "Parse error: {reason}")?;
                if let Some(help) = help {
                    write!(f, "\nHelp: {help}")?;
                }
                Ok(())
            }
        }
    }
}

impl From<dsl::Error> for Error {
    fn from(err: dsl::Error) -> Self {
        Error::Dsl(err)
    }
}

#[cfg(feature = "dsl-diag")]
impl dsl::DiagTrait for Error {
    fn span(&self) -> dsl::Span {
        match self {
            Error::Dsl(err) => err.span(),
            Error::ParseRichText(offset, err) => {
                let span = err.span();
                (span.0 + *offset, span.1 + *offset)
            }
            Error::Parse { span, .. } => *span,
        }
    }

    fn message(&self) -> String {
        match self {
            Error::Dsl(err) => err.message(),
            Error::ParseRichText(_, err) => format!("{err}"),
            Error::Parse { reason, .. } => format!("{reason}"),
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            Error::Dsl(err) => err.help(),
            Error::ParseRichText(..) => None,
            Error::Parse { help, .. } => help.clone(),
        }
    }
}

/// Parse EPLT DSL input into a list of IR figures
pub fn parse<S: AsRef<str>>(input: S) -> Result<Vec<ir::Figure>, Error> {
    let props = dsl::parse(input.as_ref().chars())?;

    let mut figs = vec![];
    for prop in props {
        if prop.name.name == "figure" {
            figs.push(parse_fig(expect_struct_val(prop)?)?);
        } else {
            return Err(Error::Parse {
                span: prop.span(),
                reason: format!("unknown top-level property: {}", prop.name.name),
                help: None,
            });
        }
    }

    Ok(figs)
}

#[cfg(feature = "dsl-diag")]
/// Parse EPLT DSL input into a list of IR figures, returning diagnostics on error.
pub fn parse_diag<'a>(
    input: &'a str,
    file_name: Option<&'a path::Path>,
) -> miette::Result<Vec<ir::Figure>> {
    match parse(input) {
        Ok(figs) => Ok(figs),
        Err(err) => {
            let src = Source {
                name: file_name.map(|s| s.to_str().unwrap_or("(non-utf8 filename)").to_string()),
                src: input.to_string(),
            };
            let diag = Diagnostic::new(Box::new(err), src);
            let report = miette::Report::new(diag);
            Err(report)
        }
    }
}

fn expect_int_scalar(scalar: ast::Scalar) -> Result<i64, Error> {
    let ast::Scalar {
        kind: ast::ScalarKind::Int(val),
        ..
    } = scalar
    else {
        return Err(Error::Parse {
            span: scalar.span,
            reason: "expected integer value".to_string(),
            help: None,
        });
    };
    Ok(val)
}

fn expect_float_val(prop: ast::Prop) -> Result<f64, Error> {
    match prop.value {
        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Float(val),
            ..
        })) => Ok(val),
        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Int(val),
            ..
        })) => Ok(val as f64),
        _ => Err(Error::Parse {
            span: prop.span(),
            reason: format!("expected float value (i.e. {}: 2.0 )", prop.name.name),
            help: None,
        }),
    }
}

fn expect_string_val(prop: ast::Prop) -> Result<(dsl::Span, String), Error> {
    let Some(ast::Value::Scalar(ast::Scalar {
        span,
        kind: ast::ScalarKind::Str(val),
    })) = prop.value
    else {
        return Err(Error::Parse {
            span: prop.span(),
            reason: format!("expected string value (i.e. {}: \"...\" )", prop.name.name),
            help: None,
        });
    };
    Ok((span, val))
}

fn expect_axis_ref_val(prop: ast::Prop) -> Result<ir::axis::Ref, Error> {
    match prop.value {
        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Str(val),
            ..
        })) => Ok(ir::axis::Ref::Id(val)),

        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Int(val),
            ..
        })) => Ok(ir::axis::Ref::Idx(val as usize)),

        _ => Err(Error::Parse {
            span: prop.span(),
            reason: format!("expected string value (i.e. {}: \"...\" )", prop.name.name),
            help: None,
        }),
    }
}

fn expect_struct_val(prop: ast::Prop) -> Result<ast::Struct, Error> {
    let Some(ast::Value::Struct(val)) = prop.value else {
        return Err(Error::Parse {
            span: prop.span(),
            reason: format!("expected struct value (i.e. {}: {{ ... }}", prop.name.name),
            help: None,
        });
    };
    Ok(val)
}

fn check_opt_type(val: &ast::Struct, type_name: &str) -> Result<(), Error> {
    if let Some(typ) = &val.typ {
        if typ.name != type_name {
            return Err(Error::Parse {
                span: typ.span,
                reason: format!(
                    "expected struct of type '{type_name}', found '{}'",
                    typ.name
                ),
                help: Some(format!(
                    "In this case, '{type_name}' can be inferred from context and is optional"
                )),
            });
        }
    }
    Ok(())
}

fn parse_rich_text(
    span: dsl::Span,
    fmt: String,
) -> Result<ParsedRichText<style::theme::Color>, Error> {
    let text = text::parse_rich_text::<style::theme::Color>(&fmt)
        .map_err(|err| Error::ParseRichText(span.0, err))?;
    Ok(text)
}

fn parse_fig(mut val: ast::Struct) -> Result<ir::Figure, Error> {
    check_opt_type(&val, "Figure")?;

    let mut row_cols: Option<(u32, u32)> = None;
    let mut plots = vec![];

    while let Some(prop) = val.take_prop("plot") {
        let (rc, plot) = parse_plot(expect_struct_val(prop)?)?;
        match (rc, &mut row_cols) {
            (None, None) => (),
            (Some(rc), Some(row_cols)) => {
                row_cols.0 = rc.0.max(row_cols.0);
                row_cols.1 = rc.1.max(row_cols.1);
            }
            (Some(rc), None) => row_cols = Some(rc),
            (None, Some(..)) => (),
        }
        plots.push((rc, plot));
    }

    let nplots = plots.len() as u32;
    while let Some(prop) = val.take_prop("subplots") {
        let span = prop.span();
        let rc = parse_subplots_val(prop.value)?;
        if let Some(row_cols) = row_cols {
            if rc.0 < row_cols.0 || rc.1 < row_cols.1 {
                return Err(Error::Parse {
                    span,
                    reason: "figure subplots value is incompatible with the plots subplot values"
                        .to_string(),
                    help: Some(
                        "You may want to only use figure subplots or only plot subplot".to_string(),
                    ),
                });
            }
        }
    }

    if row_cols.is_none() {
        row_cols = Some((nplots, 1));
    }
    let row_cols = row_cols.unwrap();

    let plots = if nplots == 1 && row_cols == (1, 1) {
        let (_, plot) = plots.into_iter().next().unwrap();
        plot.into()
    } else {
        let (rows, cols) = row_cols;
        let mut subplots = ir::Subplots::new(rows, cols);
        // eplt has rows and cols starting at 1,
        // but ir has rows and cols starting at 0
        let mut row = 0;
        let mut col = 0;
        for (rc, plot) in plots {
            let (r, c) = match rc {
                Some((r, c)) => (r - 1, c - 1),
                None => (row, col),
            };
            subplots = subplots.with_plot(r, c, plot);
            row += 1;
            if row >= rows {
                row = 0;
                col += 1;
            }
        }
        if let Some(prop) = val.take_prop("space") {
            subplots = subplots.with_space(expect_float_val(prop)? as _);
        }
        subplots.into()
    };

    let mut fig = ir::Figure::new(plots);

    for prop in val.props {
        match prop.name.name.as_str() {
            "title" => {
                let (span, fmt) = expect_string_val(prop)?;
                fig = fig.with_title(parse_rich_text(span, fmt)?.into());
            }
            "legend" => {
                fig = fig.with_legend(parse_fig_legend(prop.value)?);
            }
            // Subplots props that were not parsed for single plot
            // or stated multiple times for subplots.
            // We just ignore them.
            "cols" | "space" | "share-x" | "share-y" => (),
            _ => {
                return Err(Error::Parse {
                    span: prop.span(),
                    reason: format!("Unknown figure property: '{}'", prop.name.name),
                    help: None,
                });
            }
        }
    }

    Ok(fig)
}

fn parse_subplots_val(value: Option<ast::Value>) -> Result<(u32, u32), Error> {
    match value {
        Some(ast::Value::Seq(ast::Seq { scalars, span })) => {
            if scalars.len() == 2 {
                let mut scalars = scalars.into_iter();
                let rows = expect_int_scalar(scalars.next().unwrap())? as u32;
                let cols = expect_int_scalar(scalars.next().unwrap())? as u32;
                Ok((rows, cols))
            } else {
                Err(Error::Parse {
                    span,
                    reason: "Expected 2 values for subplot size or position".into(),
                    help: None,
                })
            }
        }
        Some(_) => Err(Error::Parse {
            span: value.as_ref().unwrap().span(),
            reason: "Could not parse subplot size or position".into(),
            help: None,
        }),
        None => Ok((1, 1)),
    }
}

fn parse_fig_legend(value: Option<ast::Value>) -> Result<ir::FigLegend, Error> {
    let mut legend = ir::FigLegend::default();

    match value {
        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Enum(ident),
            span,
        })) => match ident.as_str() {
            "Top" => legend = legend.with_pos(ir::figure::LegendPos::Top),
            "Right" => legend = legend.with_pos(ir::figure::LegendPos::Right),
            "Bottom" => legend = legend.with_pos(ir::figure::LegendPos::Bottom),
            "Left" => legend = legend.with_pos(ir::figure::LegendPos::Left),
            _ => {
                return Err(Error::Parse {
                    span,
                    reason: format!("unknown legend position: {}", ident),
                    help: None,
                });
            }
        },
        Some(_) => {
            return Err(Error::Parse {
                span: value.as_ref().unwrap().span(),
                reason: "Could not parse legend".into(),
                help: None,
            });
        }
        None => (),
    }

    Ok(legend)
}

fn parse_plot(mut val: ast::Struct) -> Result<(Option<(u32, u32)>, ir::plot::Plot), Error> {
    check_opt_type(&val, "Plot")?;

    let mut series = vec![];
    loop {
        let Some(prop) = val.take_prop("series") else {
            break;
        };
        series.push(parse_series(expect_struct_val(prop)?)?);
    }
    let mut row_cols = None;
    let mut plot = ir::Plot::new(series);

    for prop in val.props {
        match prop.name.name.as_str() {
            "subplot" => {
                row_cols = Some(parse_subplots_val(prop.value)?);
            }
            "x-axis" => plot = plot.with_x_axis(parse_axis(prop, false)?),
            "y-axis" => plot = plot.with_y_axis(parse_axis(prop, true)?),
            "title" => plot = plot.with_title(expect_string_val(prop)?.1.into()),
            "legend" => plot = plot.with_legend(parse_plot_legend(prop.value)?),
            _ => {
                return Err(Error::Parse {
                    span: prop.span(),
                    reason: format!("Unknown plot property: '{}'", prop.name.name),
                    help: None,
                });
            }
        }
    }

    Ok((row_cols, plot))
}

fn parse_plot_legend(value: Option<ast::Value>) -> Result<ir::plot::PlotLegend, Error> {
    let mut legend = ir::plot::PlotLegend::default();

    match value {
        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Enum(ident),
            span,
        })) => match ident.as_str() {
            "OutTop" | "Top" => legend = legend.with_pos(ir::plot::LegendPos::OutTop),
            "OutRight" | "Right" => legend = legend.with_pos(ir::plot::LegendPos::OutRight),
            "OutBottom" | "Bottom" => legend = legend.with_pos(ir::plot::LegendPos::OutBottom),
            "OutLeft" | "Left" => legend = legend.with_pos(ir::plot::LegendPos::OutLeft),
            "InTop" => legend = legend.with_pos(ir::plot::LegendPos::InTop),
            "InTopRight" => legend = legend.with_pos(ir::plot::LegendPos::InTopRight),
            "InRight" => legend = legend.with_pos(ir::plot::LegendPos::InRight),
            "InBottomRight" => legend = legend.with_pos(ir::plot::LegendPos::InBottomRight),
            "InBottom" => legend = legend.with_pos(ir::plot::LegendPos::InBottom),
            "InBottomLeft" => legend = legend.with_pos(ir::plot::LegendPos::InBottomLeft),
            "InLeft" => legend = legend.with_pos(ir::plot::LegendPos::InLeft),
            "InTopLeft" => legend = legend.with_pos(ir::plot::LegendPos::InTopLeft),
            _ => {
                return Err(Error::Parse {
                    span,
                    reason: format!("unknown legend position: {}", ident),
                    help: None,
                });
            }
        },
        Some(_) => {
            return Err(Error::Parse {
                span: value.as_ref().unwrap().span(),
                reason: "Could not parse legend".into(),
                help: None,
            });
        }
        None => (),
    }

    Ok(legend)
}

fn parse_series(val: ast::Struct) -> Result<ir::Series, Error> {
    let Some(ident) = &val.typ else {
        return Err(Error::Parse {
            span: val.span,
            reason: "series type can't be inferred and must be speficied".into(),
            help: None,
        });
    };

    match ident.name.as_str() {
        "Line" => Ok(parse_line(val)?.into()),
        "Scatter" => Ok(parse_scatter(val)?.into()),
        "Histogram" => Ok(parse_histogram(val)?.into()),
        "Bars" => Ok(parse_bars(val)?.into()),
        "BarsGroup" => Ok(parse_bars_group(val)?.into()),
        _ => Err(Error::Parse {
            span: ident.span,
            reason: format!("unknown series type: {}", ident.name),
            help: None,
        }),
    }
}

fn expect_prop(val: &mut ast::Struct, name: &str) -> Result<ast::Prop, Error> {
    val.take_prop(name).ok_or(Error::Parse {
        span: val.span,
        reason: format!("expected '{name}' property"),
        help: None,
    })
}

fn expect_data_prop(val: &mut ast::Struct, prop_name: &str) -> Result<ir::DataCol, Error> {
    let prop = expect_prop(val, prop_name)?;
    match prop.value {
        Some(ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Str(val),
            ..
        })) => Ok(ir::DataCol::SrcRef(val)),
        Some(ast::Value::Array(ast::Array {
            kind: ast::ArrayKind::Int(vals),
            ..
        })) => Ok(ir::DataCol::Inline(vals.into())),
        Some(ast::Value::Array(ast::Array {
            kind: ast::ArrayKind::Float(vals),
            ..
        })) => Ok(ir::DataCol::Inline(vals.into())),
        Some(ast::Value::Array(ast::Array {
            kind: ast::ArrayKind::Str(vals),
            ..
        })) => Ok(ir::DataCol::Inline(vals.into())),
        _ => Err(Error::Parse {
            span: prop.span(),
            reason: format!("Could not parse '{prop_name}' as a data column"),
            help: None,
        }),
    }
}

fn parse_line(mut val: ast::Struct) -> Result<ir::series::Line, Error> {
    let x_data = expect_data_prop(&mut val, "x-data")?;
    let y_data = expect_data_prop(&mut val, "y-data")?;

    let mut line = ir::series::Line::new(x_data, y_data);

    if let Some(prop) = val.take_prop("name") {
        line = line.with_name(expect_string_val(prop)?.1);
    }
    if let Some(prop) = val.take_prop("x-axis") {
        line = line.with_x_axis(expect_axis_ref_val(prop)?);
    }
    if let Some(prop) = val.take_prop("y-axis") {
        line = line.with_y_axis(expect_axis_ref_val(prop)?);
    }

    Ok(line)
}

fn parse_scatter(mut val: ast::Struct) -> Result<ir::series::Scatter, Error> {
    let x_data = expect_data_prop(&mut val, "x-data")?;
    let y_data = expect_data_prop(&mut val, "y-data")?;

    let mut series = ir::series::Scatter::new(x_data, y_data);

    if let Some(prop) = val.take_prop("name") {
        series = series.with_name(expect_string_val(prop)?.1);
    }
    if let Some(prop) = val.take_prop("x-axis") {
        series = series.with_x_axis(expect_axis_ref_val(prop)?);
    }
    if let Some(prop) = val.take_prop("y-axis") {
        series = series.with_y_axis(expect_axis_ref_val(prop)?);
    }

    Ok(series)
}

fn parse_histogram(mut val: ast::Struct) -> Result<ir::series::Histogram, Error> {
    let data = expect_data_prop(&mut val, "data")?;

    let mut series = ir::series::Histogram::new(data);

    if let Some(prop) = val.take_prop("name") {
        series = series.with_name(expect_string_val(prop)?.1);
    }
    if let Some(prop) = val.take_prop("x-axis") {
        series = series.with_x_axis(expect_axis_ref_val(prop)?);
    }
    if let Some(prop) = val.take_prop("y-axis") {
        series = series.with_y_axis(expect_axis_ref_val(prop)?);
    }

    Ok(series)
}

fn parse_bars(mut val: ast::Struct) -> Result<ir::series::Bars, Error> {
    let x_data = expect_data_prop(&mut val, "x-data")?;
    let y_data = expect_data_prop(&mut val, "y-data")?;

    let mut bars = ir::series::Bars::new(x_data, y_data);

    if let Some(prop) = val.take_prop("name") {
        bars = bars.with_name(expect_string_val(prop)?.1);
    }

    Ok(bars)
}

fn parse_bars_group(_val: ast::Struct) -> Result<ir::series::BarsGroup, Error> {
    todo!()
}

fn parse_axis(prop: ast::Prop, is_y: bool) -> Result<ir::Axis, Error> {
    let Some(val) = prop.value else {
        return Ok(Default::default());
    };
    match val {
        ast::Value::Scalar(ast::Scalar {
            span,
            kind: ast::ScalarKind::Str(title),
        }) => Ok(ir::Axis::default().with_title(parse_rich_text(span, title)?.into())),

        ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Enum(ident),
            span,
        }) => axis_set_enum_field(Default::default(), is_y, span, ident.as_str()),

        ast::Value::Seq(seq) => parse_axis_seq(seq, is_y),

        ast::Value::Struct(val) => parse_axis_struct(val, is_y),

        _ => Err(Error::Parse {
            span: val.span(),
            reason: "Could not parse axis".into(),
            help: None,
        }),
    }
}

fn axis_set_enum_field(
    axis: ir::Axis,
    is_y: bool,
    span: dsl::Span,
    ident: &str,
) -> Result<ir::Axis, Error> {
    match ident {
        "LogScale" => Ok(axis.with_scale(ir::axis::LogScale::default().into())),
        "Ticks" => Ok(axis.with_ticks(Default::default())),
        "PiMultipleTicks" => Ok(axis.with_ticks(
            ir::axis::Ticks::default()
                .with_locator(ir::axis::ticks::Locator::PiMultiple { bins: 9 }),
        )),
        "MinorTicks" => Ok(axis.with_minor_ticks(Default::default())),
        "Grid" => Ok(axis.with_grid(Default::default())),
        "MinorGrid" => Ok(axis.with_minor_grid(Default::default())),
        "MainSide" | "OppositeSide" | "LeftSide" | "RightSide" | "TopSide" | "BottomSide" => {
            axis_set_side_enum(axis, is_y, span, ident)
        }
        _ => Err(Error::Parse {
            span,
            reason: format!("unknown axis property enum: {}", ident),
            help: None,
        }),
    }
}

fn axis_set_side_enum(
    axis: ir::Axis,
    is_y: bool,
    span: dsl::Span,
    ident: &str,
) -> Result<ir::Axis, Error> {
    match ident {
        "MainSide" => Ok(axis),
        "OppositeSide" => Ok(axis.with_opposite_side()),
        "LeftSide" if is_y => Ok(axis),
        "RightSide" if is_y => Ok(axis.with_opposite_side()),
        "TopSide" if !is_y => Ok(axis.with_opposite_side()),
        "BottomSide" if !is_y => Ok(axis),
        "LeftSide" | "RightSide" if !is_y => Err(Error::Parse {
            span,
            reason: format!("axis side '{}' is invalid for x-axis", ident),
            help: Some("Valid enums are BottomSide and MainSide (default) as well as TopSide and OppositeSide".into()),
        }),
        "TopSide" | "BottomSide" if is_y => Err(Error::Parse {
            span,
            reason: format!("axis side '{}' is invalid for y-axis", ident),
            help: Some("Valid enums are LeftSide and MainSide (default) as well as RightSide and OppositeSide".into()),
        }),
        _ => unreachable!(),
    }
}

fn axis_set_side_prop(
    axis: ir::Axis,
    is_y: bool,
    span: dsl::Span,
    ident: &str,
) -> Result<ir::Axis, Error> {
    match ident {
        "main-side" => Ok(axis),
        "opposote-side" => Ok(axis.with_opposite_side()),
        "left-side" if is_y => Ok(axis),
        "right-side" if is_y => Ok(axis.with_opposite_side()),
        "top-side" if !is_y => Ok(axis.with_opposite_side()),
        "bottom-side" if !is_y => Ok(axis),
        "left-side" | "right-side" if !is_y => Err(Error::Parse {
            span,
            reason: format!("axis property '{}' is invalid for x-axis", ident),
            help: Some("Valid side properties are bottom-side or main-side (default) as well as top-side or opposite-side".into()),
        }),
        "top-side" | "bottom-side" if is_y => Err(Error::Parse {
            span,
            reason: format!("axis property '{}' is invalid for y-axis", ident),
            help: Some("Valid side properties are left-side or main-side (default) as well as right-side or opposite-side".into()),
        }),
        _ => unreachable!(),
    }
}

fn parse_axis_seq(seq: ast::Seq, is_y: bool) -> Result<ir::Axis, Error> {
    let mut axis = ir::Axis::default();
    for scalar in seq.scalars {
        match scalar {
            ast::Scalar {
                span,
                kind: ast::ScalarKind::Str(title),
            } => axis = axis.with_title(parse_rich_text(span, title)?.into()),
            ast::Scalar {
                kind: ast::ScalarKind::Enum(ident),
                span,
            } => axis = axis_set_enum_field(axis, is_y, span, ident.as_str())?,
            ast::Scalar {
                kind: ast::ScalarKind::Func(ast::Func { name, args }),
                span,
            } => {
                let mut args_iter = args.scalars.into_iter();
                let arg1 = args_iter.next();
                if name.name == "id" {
                    let id = match arg1 {
                        Some(ast::Scalar {
                            kind: ast::ScalarKind::Str(id),
                            ..
                        }) => id,
                        _ => {
                            return Err(Error::Parse {
                                span,
                                reason: "Could not parse axis id".into(),
                                help: Some("Expected a single string argument".to_string()),
                            });
                        }
                    };
                    axis = axis.with_id(id);
                } else if name.name == "shared" {
                    let ax_ref = match arg1 {
                        Some(ast::Scalar {
                            kind: ast::ScalarKind::Str(id),
                            ..
                        }) => ir::axis::Ref::Id(id),
                        Some(ast::Scalar {
                            kind: ast::ScalarKind::Int(idx),
                            ..
                        }) => ir::axis::Ref::Idx(idx as usize),
                        _ => {
                            return Err(Error::Parse {
                                span,
                                reason: "Could not parse axis shared reference".into(),
                                help: Some(
                                    "Expected a single string or integer argument".to_string(),
                                ),
                            });
                        }
                    };
                    axis = axis.with_scale(ir::axis::Scale::Shared(ax_ref));
                } else {
                    return Err(Error::Parse {
                        span,
                        reason: "Unknown axis attribute".into(),
                        help: None,
                    });
                }
                if args_iter.next().is_some() {
                    return Err(Error::Parse {
                        span,
                        reason: format!("Too many arguments for {}", name.name),
                        help: None,
                    });
                }
            }
            _ => {
                return Err(Error::Parse {
                    span: seq.span,
                    reason: "Could not parse axis".into(),
                    help: None,
                });
            }
        }
    }
    Ok(axis)
}

fn parse_axis_struct(val: ast::Struct, is_y: bool) -> Result<ir::Axis, Error> {
    check_opt_type(&val, "Axis")?;
    let mut axis = ir::Axis::default();
    for prop in val.props {
        match prop.name.name.as_str() {
            "title" => {
                let (span, title) = expect_string_val(prop)?;
                axis = axis.with_title(parse_rich_text(span, title)?.into());
            }
            "ticks" => {
                axis = axis.with_ticks(parse_ticks(prop)?);
            }
            "minor-ticks" => {
                axis = axis.with_minor_ticks(Default::default());
            }
            "grid" => {
                axis = axis.with_grid(Default::default());
            }
            "minor-grid" => {
                axis = axis.with_minor_grid(Default::default());
            }
            "main-side" | "opposite-side" | "left-side" | "right-side" | "top-side"
            | "bottom-side" => {
                axis = axis_set_side_prop(axis, is_y, prop.span(), prop.name.name.as_str())?;
            }
            _ => {
                return Err(Error::Parse {
                    span: prop.span(),
                    reason: format!("unknown axis property: {}", prop.name.name),
                    help: None,
                });
            }
        }
    }
    Ok(axis)
}

fn parse_ticks(prop: ast::Prop) -> Result<ir::axis::Ticks, Error> {
    let Some(val) = prop.value else {
        return Ok(Default::default());
    };
    match val {
        ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Enum(ident),
            span,
        }) => Ok(ticks_set_enum_field(
            ir::axis::Ticks::default(),
            span,
            &ident,
        )?),
        ast::Value::Seq(val) => parse_ticks_seq(val),
        ast::Value::Struct(val) => parse_ticks_struct(val),
        _ => Err(Error::Parse {
            span: val.span(),
            reason: "Could not parse ticks".into(),
            help: None,
        }),
    }
}

fn parse_ticks_seq(val: ast::Seq) -> Result<ir::axis::Ticks, Error> {
    let mut ticks = ir::axis::Ticks::default();
    for scalar in val.scalars {
        match scalar {
            ast::Scalar {
                kind: ast::ScalarKind::Enum(ident),
                span,
            } => ticks = ticks_set_enum_field(ticks, span, ident.as_str())?,
            _ => {
                return Err(Error::Parse {
                    span: val.span,
                    reason: "Could not parse ticks".into(),
                    help: None,
                });
            }
        }
    }
    Ok(ticks)
}

fn ticks_set_enum_field(
    ticks: ir::axis::Ticks,
    span: dsl::Span,
    ident: &str,
) -> Result<ir::axis::Ticks, Error> {
    match ident {
        "Locator" => Ok(ticks.with_locator(ir::axis::ticks::Locator::default())),
        "PiMultiple" => Ok(ticks.with_locator(ir::axis::ticks::Locator::PiMultiple { bins: 9 })),
        _ => Err(Error::Parse {
            span,
            reason: format!("unknown ticks property enum: {}", ident),
            help: None,
        }),
    }
}

fn parse_ticks_struct(val: ast::Struct) -> Result<ir::axis::Ticks, Error> {
    check_opt_type(&val, "Ticks")?;
    let mut ticks = ir::axis::Ticks::default();
    for prop in val.props {
        match prop.name.name.as_str() {
            "locator" => {
                ticks = ticks.with_locator(ir::axis::ticks::Locator::default());
            }
            _ => {
                return Err(Error::Parse {
                    span: prop.span(),
                    reason: format!("unknown ticks property: {}", prop.name.name),
                    help: None,
                });
            }
        }
    }
    Ok(ticks)
}
