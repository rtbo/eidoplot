use eidoplot_base::{ColorU8, geom};
use ttf_parser as ttf;

use super::RichText;
use crate::{BBox, font, fontdb};

#[derive(Debug)]
pub enum RichPrimitive<'a> {
    Fill(&'a geom::Path, ColorU8),
    Stroke(&'a geom::Path, ColorU8, f32),
}

pub fn render_rich_text_with<RenderFn>(
    text: &RichText,
    fontdb: &fontdb::Database,
    mut render_fn: RenderFn,
) -> Result<(), ttf::FaceParsingError>
where
    RenderFn: FnMut(RichPrimitive<'_>),
{
    let mut span_builder = geom::PathBuilder::new();
    let mut glyph_builder = geom::PathBuilder::new();

    for line in &text.lines {
        for shape in &line.shapes {
            (glyph_builder, span_builder) = fontdb
                .with_face_data(shape.face_id, |data, index| {
                    let mut face = ttf::Face::parse(data, index).unwrap();
                    font::apply_ttf_variations(&mut face, shape.font());

                    // TODO: get span bbox and render underline and strikeout lines

                    for span in &shape.spans {
                        for glyph in shape
                            .glyphs
                            .iter()
                            .filter(|g| g.cluster >= span.start && g.cluster < span.end)
                        {
                            {
                                let mut builder = crate::Outliner(&mut glyph_builder);
                                face.outline_glyph(glyph.id, &mut builder);
                            }

                            if let Some(path) = glyph_builder.finish() {
                                let path = path.transform(glyph.ts).unwrap();
                                span_builder.push_path(&path);

                                glyph_builder = path.clear();
                            } else {
                                glyph_builder = geom::PathBuilder::new();
                            }
                        }

                        if span.props.underline {
                            let line = shape.metrics.uline;
                            let path = line_path(span.bbox, shape.y_baseline, line, glyph_builder);
                            span_builder.push_path(&path);
                            glyph_builder = path.clear();
                        }
                        if span.props.strikeout {
                            let line = shape.metrics.strikeout;
                            let path = line_path(span.bbox, shape.y_baseline, line, glyph_builder);
                            span_builder.push_path(&path);
                            glyph_builder = path.clear();
                        }

                        if let Some(path) = span_builder.finish() {
                            if let Some(c) = span.props.fill.as_ref() {
                                let prim = RichPrimitive::Fill(&path, *c);
                                render_fn(prim);
                            }
                            if let Some((c, thickness)) = span.props.outline.as_ref() {
                                let prim = RichPrimitive::Stroke(&path, *c, *thickness);
                                render_fn(prim);
                            }
                            span_builder = path.clear();
                        } else {
                            span_builder = geom::PathBuilder::new();
                        }
                    }

                    (glyph_builder, span_builder)
                })
                .unwrap();
        }
    }

    Ok(())
}

pub fn render_rich_text(
    text: &RichText,
    fontdb: &fontdb::Database,
    transform: geom::Transform,
    mask: Option<&tiny_skia::Mask>,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) -> Result<(), ttf::FaceParsingError> {
    let render_fn = |primitive: RichPrimitive| match primitive {
        RichPrimitive::Fill(path, color) => {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
            pixmap.fill_path(path, &paint, tiny_skia::FillRule::Winding, transform, mask);
        }
        RichPrimitive::Stroke(path, color, width) => {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
            let mut stroke = tiny_skia::Stroke::default();
            stroke.width = width;
            pixmap.stroke_path(path, &paint, &stroke, transform, mask);
        }
    };
    render_rich_text_with(text, fontdb, render_fn)
}

fn line_path(
    bbox: BBox,
    y_baseline: f32,
    line: font::ScaledLineMetrics,
    mut builder: geom::PathBuilder,
) -> geom::Path {
    // there is no y-flip transform on this one
    builder.move_to(bbox.left, y_baseline - line.position);
    builder.line_to(bbox.right, y_baseline - line.position);
    builder.line_to(bbox.right, y_baseline - line.position + line.thickness);
    builder.line_to(bbox.left, y_baseline - line.position + line.thickness);
    builder.close();
    builder.finish().unwrap()
}
