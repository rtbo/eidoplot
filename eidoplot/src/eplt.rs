use std::fmt;

use eidoplot_dsl::{self as dsl, ast};

use crate::ir;

#[derive(Debug, Clone)]
pub enum Error {
    Dsl(dsl::Error),
    Parse(dsl::Span, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Dsl(err) => err.fmt(f),
            Error::Parse(_, reason) => write!(f, "Parse error: {reason}"),
        }
    }
}

impl From<dsl::Error> for Error {
    fn from(err: dsl::Error) -> Self {
        Error::Dsl(err)
    }
}

pub fn parse_eplt<S: AsRef<str>>(input: S) -> Result<Vec<ir::Figure>, Error> {
    let props = dsl::parse(input.as_ref().chars())?;

    let mut figs = vec![];

    for prop in props {
        if prop.name.name == "figure" {
            figs.push(parse_fig(expect_struct_val(prop)?)?);
        }
    }

    Ok(figs)
}

fn expect_string_val(prop: ast::Prop) -> Result<String, Error> {
    let Some(ast::Value::Scalar(ast::Scalar {
        kind: ast::ScalarKind::Str(val),
        ..
    })) = prop.value
    else {
        return Err(Error::Parse(
            prop.span(),
            format!("expected string value (i.e. {}: \"...\" )", prop.name.name),
        ));
    };
    Ok(val)
}

fn expect_struct_val(prop: ast::Prop) -> Result<ast::Struct, Error> {
    let Some(ast::Value::Struct(val)) = prop.value else {
        return Err(Error::Parse(
            prop.span(),
            format!("expected struct value (i.e. {}: {{ ... }}", prop.name.name),
        ));
    };
    Ok(val)
}

fn check_opt_type(val: &ast::Struct, type_name: &str) -> Result<(), Error> {
    if let Some(typ) = &val.typ {
        if typ.name != type_name {
            return Err(Error::Parse(
                typ.span,
                format!(
                    "expected struct of type '{type_name}', found '{}'.\n",
                    typ.name
                ) + &format!(
                    "In this case, '{type_name}' can be inferred from context and not be specified at all."
                ),
            ));
        }
    }
    Ok(())
}

fn parse_fig(val: ast::Struct) -> Result<ir::Figure, Error> {
    let plot_def_count = val.has_prop("plot") as u8 + val.has_prop("subplots") as u8;
    if plot_def_count != 1 {
        return Err(Error::Parse(
            val.span,
            "figure must have exactly one of 'plot' or 'subplots' property".into(),
        ));
    }

    check_opt_type(&val, "Figure")?;

    let mut val = val;

    let plots = if let Some(prop) = val.take_prop("plot") {
        ir::figure::Plots::Plot(parse_plot(expect_struct_val(prop)?)?)
    } else if let Some(prop) = val.take_prop("subplots") {
        ir::figure::Plots::Subplots(parse_subplots(expect_struct_val(prop)?)?)
    } else {
        unreachable!()
    };

    let mut fig = ir::Figure::new(plots);

    if let Some(prop) = val.take_prop("title") {
        // TODO: parse also as struct with font options, or seq with color
        fig = fig.with_title(expect_string_val(prop)?.into());
    }

    Ok(fig)
}

fn parse_subplots(val: ast::Struct) -> Result<ir::figure::Subplots, Error> {
    check_opt_type(&val, "Subplots")?;
    todo!()
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

    if let Some(prop) = val.take_prop("x-axis") {
        plot = plot.with_x_axis(parse_axis(prop)?);
    }
    if let Some(prop) = val.take_prop("y-axis") {
        plot = plot.with_y_axis(parse_axis(prop)?);
    }

    if let Some(prop) = val.take_prop("title") {
        plot = plot.with_title(expect_string_val(prop)?.into());
    }

    if let Some(prop) = val.take_prop("legend") {
        plot = plot.with_legend(parse_plot_legend(prop.value)?);
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
                return Err(Error::Parse(
                    span,
                    format!("unknown legend position: {}", ident),
                ));
            }
        },
        Some(_) => {
            return Err(Error::Parse(
                value.as_ref().unwrap().span(),
                "Could not parse legend".into(),
            ));
        }
        None => (),
    }

    Ok(legend)
}

fn parse_series(val: ast::Struct) -> Result<ir::Series, Error> {
    let Some(ident) = &val.typ else {
        return Err(Error::Parse(
            val.span,
            "series type can't be inferred and must be speficied".into(),
        ));
    };

    match ident.name.as_str() {
        "Line" => Ok(parse_line(val)?.into()),
        "Scatter" => Ok(parse_scatter(val)?.into()),
        "Histogram" => Ok(parse_histogram(val)?.into()),
        "Bars" => Ok(parse_bars(val)?.into()),
        "BarsGroup" => Ok(parse_bars_group(val)?.into()),
        _ => Err(Error::Parse(
            ident.span,
            format!("unknown series type: {}", ident.name),
        )),
    }
}

fn expect_prop(val: &mut ast::Struct, name: &str) -> Result<ast::Prop, Error> {
    val.take_prop(name).ok_or(Error::Parse(
        val.span,
        format!("expected '{name}' property"),
    ))
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
        _ => Err(Error::Parse(
            prop.span(),
            format!("Could not parse '{prop_name}' as a data column"),
        )),
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

        _ => Err(Error::Parse(val.span(), "Could not parse axis".into())),
    }
}

fn axis_set_enum_field(axis: ir::Axis, span: dsl::Span, ident: &str) -> Result<ir::Axis, Error> {
    match ident {
        "Ticks" => Ok(axis.with_ticks(Default::default())),
        "MinorTicks" => Ok(axis.with_minor_ticks(Default::default())),
        "Grid" => Ok(axis.with_grid(Default::default())),
        "MinorGrid" => Ok(axis.with_minor_grid(Default::default())),
        _ => Err(Error::Parse(
            span,
            format!("unknown axis property enum: {}", ident),
        )),
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
            _ => return Err(Error::Parse(seq.span, "Could not parse axis".into())),
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
                axis = axis.with_ticks(Default::default());
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
                return Err(Error::Parse(
                    prop.span(),
                    format!("unknown axis property: {}", prop.name.name),
                ));
            }
        }
    }
    Ok(axis)
}
