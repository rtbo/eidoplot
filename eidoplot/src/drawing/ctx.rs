use std::sync::Arc;

use rustybuzz::ttf_parser;

use crate::{ir, missing_params};

pub struct Ctx<'a, S> {
    pub surface: &'a mut S,
    pub fontdb: Arc<fontdb::Database>,
}

impl<'a, S> Ctx<'a, S> {
    pub fn calculate_x_axis_height(&self, x_axis: &ir::Axis) -> f32 {
        let mut height = 0.0;
        if let Some(ticks) = &x_axis.ticks {
            height +=
                missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + ticks.font().size();
        }
        if x_axis.label.is_some() {
            height +=
                2.0 * missing_params::AXIS_LABEL_MARGIN + missing_params::AXIS_LABEL_FONT_SIZE;
        }
        height
    }

    // TODO: When pxl draws on its own rather than using resvg,
    // this function should return the calculated shapes and cache them in the render::Text
    // and send them to the surface for reuse
    pub fn calculate_y_axis_width(
        &self,
        y_axis: &ir::Axis,
        y_ticks: Option<&[(f64, String)]>,
    ) -> f32 {
        let mut width = 0.0;
        if y_axis.label.is_some() {
            width += 2.0 * missing_params::AXIS_LABEL_MARGIN + missing_params::AXIS_LABEL_FONT_SIZE;
        }
        if let Some(ticks) = y_ticks {
            let font = y_axis.ticks.as_ref().unwrap().font();
            let query = fontdb::Query {
                families: &[fontdb::Family::SansSerif],
                ..Default::default()
            };
            // FIXME: error mgmt
            let id = self.fontdb.query(&query).expect("Should find a face");
            let max_w = self
                .fontdb
                .with_face_data(id, |data, index| {
                    let face = ttf_parser::Face::parse(data, index).unwrap();
                    let units_per_em = face.units_per_em() as f32;
                    let hbf = rustybuzz::Face::from_face(face);
                    let scale = font.size() / units_per_em;
                    let mut max_w = f32::NAN;
                    for (_, label) in ticks {
                        let mut buffer = rustybuzz::UnicodeBuffer::new();
                        buffer.push_str(label);
                        let glyph_buffer = rustybuzz::shape(&hbf, &[], buffer);
                        let w: i32 = glyph_buffer
                            .glyph_positions()
                            .iter()
                            .map(|gp| gp.x_advance)
                            .sum();
                        max_w = max_w.max(w as f32 * scale);
                    }
                    max_w
                })
                .expect("Should find face data");
            width += missing_params::TICK_SIZE + missing_params::TICK_LABEL_MARGIN + max_w;
        }
        width
    }
}
