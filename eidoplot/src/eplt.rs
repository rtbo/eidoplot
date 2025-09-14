use std::fmt;
#[cfg(feature = "dsl-diag")]
use std::path;

use eidoplot_dsl::{self as dsl, ast};

use crate::ir;

#[cfg(feature = "dsl-diag")]
pub use dsl::{Diagnostic, Source};

#[derive(Debug, Clone)]
pub enum Error {
    Dsl(dsl::Error),
    Parse {
        span: dsl::Span,
        reason: String,
        help: Option<String>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Dsl(err) => err.fmt(f),
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
            Error::Parse { span, .. } => *span,
        }
    }

    fn message(&self) -> String {
        match self {
            Error::Dsl(err) => err.message(),
            Error::Parse { reason, .. } => format!("{reason}"),
        }
    }

    fn help(&self) -> Option<String> {
        match self {
            Error::Dsl(err) => err.help(),
            Error::Parse { help, .. } => help.clone(),
        }
    }
}

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

fn expect_int_val(prop: ast::Prop) -> Result<i64, Error> {
    let Some(ast::Value::Scalar(ast::Scalar {
        kind: ast::ScalarKind::Int(val),
        ..
    })) = prop.value
    else {
        return Err(Error::Parse {
            span: prop.span(),
            reason: format!("expected integer value (i.e. {}: 2 )", prop.name.name),
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

fn expect_string_val(prop: ast::Prop) -> Result<String, Error> {
    let Some(ast::Value::Scalar(ast::Scalar {
        kind: ast::ScalarKind::Str(val),
        ..
    })) = prop.value
    else {
        return Err(Error::Parse {
            span: prop.span(),
            reason: format!("expected string value (i.e. {}: \"...\" )", prop.name.name),
            help: None,
        });
    };
    Ok(val)
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

fn check_empty_bool_val(prop: &ast::Prop) -> Result<(), Error> {
    if prop.value.is_some() {
        return Err(Error::Parse {
            span: prop.span(),
            reason: format!(
                "boolean property '{}' doesn't expect a value",
                prop.name.name
            ),
            help: Some(format!(
                "To set true, only state the name '{}'. To set false, omit the property.",
                prop.name.name
            )),
        });
    }
    Ok(())
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

fn parse_fig(mut val: ast::Struct) -> Result<ir::Figure, Error> {
    check_opt_type(&val, "Figure")?;

    let mut plots = vec![];
    while let Some(prop) = val.take_prop("plot") {
        plots.push(parse_plot(expect_struct_val(prop)?)?);
    }

    let plots = if plots.len() == 1 {
        let plot = plots.into_iter().next().unwrap();
        plot.into()
    } else {
        let mut subplots = ir::Subplots::new(plots);
        if let Some(prop) = val.take_prop("cols") {
            subplots = subplots.with_cols(expect_int_val(prop)? as _);
        }
        if let Some(prop) = val.take_prop("space") {
            subplots = subplots.with_space(expect_float_val(prop)? as _);
        }
        if let Some(prop) = val.take_prop("share-x") {
            check_empty_bool_val(&prop)?;
            subplots = subplots.with_share_x();
        }
        if let Some(prop) = val.take_prop("share-y") {
            check_empty_bool_val(&prop)?;
            subplots = subplots.with_share_y();
        }
        subplots.into()
    };

    let mut fig = ir::Figure::new(plots);

    for prop in val.props {
        match prop.name.name.as_str() {
            "title" => fig = fig.with_title(expect_string_val(prop)?.into()),
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

fn parse_plot(mut val: ast::Struct) -> Result<ir::plot::Plot, Error> {
    check_opt_type(&val, "Plot")?;

    let mut series = vec![];
    loop {
        let Some(prop) = val.take_prop("series") else {
            break;
        };
        series.push(parse_series(expect_struct_val(prop)?)?);
    }
    let mut plot = ir::Plot::new(series);

    for prop in val.props {
        match prop.name.name.as_str() {
            "x-axis" => plot = plot.with_x_axis(parse_axis(prop)?),
            "y-axis" => plot = plot.with_y_axis(parse_axis(prop)?),
            "title" => plot = plot.with_title(expect_string_val(prop)?.into()),
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

    Ok(plot)
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
        line = line.with_name(expect_string_val(prop)?.into());
    }

    Ok(line)
}

fn parse_scatter(mut val: ast::Struct) -> Result<ir::series::Scatter, Error> {
    let x_data = expect_data_prop(&mut val, "x-data")?;
    let y_data = expect_data_prop(&mut val, "y-data")?;

    let mut scatter = ir::series::Scatter::new(x_data, y_data);

    if let Some(prop) = val.take_prop("name") {
        scatter = scatter.with_name(expect_string_val(prop)?.into());
    }

    Ok(scatter)
}

fn parse_histogram(mut val: ast::Struct) -> Result<ir::series::Histogram, Error> {
    let data = expect_data_prop(&mut val, "data")?;

    let mut histogram = ir::series::Histogram::new(data);

    if let Some(prop) = val.take_prop("name") {
        histogram = histogram.with_name(expect_string_val(prop)?.into());
    }

    Ok(histogram)
}

fn parse_bars(mut val: ast::Struct) -> Result<ir::series::Bars, Error> {
    let x_data = expect_data_prop(&mut val, "x-data")?;
    let y_data = expect_data_prop(&mut val, "y-data")?;

    let mut bars = ir::series::Bars::new(x_data, y_data);

    if let Some(prop) = val.take_prop("name") {
        bars = bars.with_name(expect_string_val(prop)?.into());
    }

    Ok(bars)
}

fn parse_bars_group(_val: ast::Struct) -> Result<ir::series::BarsGroup, Error> {
    todo!()
}

fn parse_axis(prop: ast::Prop) -> Result<ir::Axis, Error> {
    let Some(val) = prop.value else {
        return Ok(Default::default());
    };
    match val {
        ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Str(title),
            ..
        }) => Ok(ir::Axis::default().with_title(title.into())),

        ast::Value::Scalar(ast::Scalar {
            kind: ast::ScalarKind::Enum(ident),
            span,
        }) => axis_set_enum_field(Default::default(), span, ident.as_str()),

        ast::Value::Seq(seq) => parse_axis_seq(seq),

        ast::Value::Struct(val) => parse_axis_struct(val),

        _ => Err(Error::Parse {
            span: val.span(),
            reason: "Could not parse axis".into(),
            help: None,
        }),
    }
}

fn axis_set_enum_field(axis: ir::Axis, span: dsl::Span, ident: &str) -> Result<ir::Axis, Error> {
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
        _ => Err(Error::Parse {
            span,
            reason: format!("unknown axis property enum: {}", ident),
            help: None,
        }),
    }
}

fn parse_axis_seq(seq: ast::Seq) -> Result<ir::Axis, Error> {
    let mut axis = ir::Axis::default();
    for scalar in seq.scalars {
        match scalar {
            ast::Scalar {
                kind: ast::ScalarKind::Str(title),
                ..
            } => axis = axis.with_title(title.into()),
            ast::Scalar {
                kind: ast::ScalarKind::Enum(ident),
                span,
            } => axis = axis_set_enum_field(axis, span, ident.as_str())?,
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

fn parse_axis_struct(val: ast::Struct) -> Result<ir::Axis, Error> {
    check_opt_type(&val, "Axis")?;
    let mut axis = ir::Axis::default();
    for prop in val.props {
        match prop.name.name.as_str() {
            "title" => {
                axis = axis.with_title(expect_string_val(prop)?.into());
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
