
use crate::{geom, style};

const SQRT2: f32 = 1.41421356237;

pub fn marker_path<C: style::Color>(marker: &style::Marker<C>) -> geom::Path {
    match marker.shape {
        style::MarkerShape::Circle => {
            let radius = marker.size.0 / 2.0;
            geom::PathBuilder::from_circle(0.0, 0.0, radius)
                .expect("Should be a valid path")
        }
        style::MarkerShape::Square => {
            let half_w = marker.size.0 / 2.0;
            let half_h = marker.size.0 / 2.0;
            let mut builder = geom::PathBuilder::new();
            builder.move_to(-half_w, -half_h);
            builder.line_to(half_w, -half_h);
            builder.line_to(half_w, half_h);
            builder.line_to(-half_w, half_h);
            builder.close();
            builder.finish().expect("Should be a valid path")
        }
        style::MarkerShape::Diamond => {
            let half_w = marker.size.0 / SQRT2;
            let half_h = marker.size.0 / SQRT2;
            let mut builder = geom::PathBuilder::new();
            builder.move_to(0.0, -half_h);
            builder.line_to(half_w, 0.0);
            builder.line_to(0.0, half_h);
            builder.line_to(-half_w, 0.0);
            builder.close();
            builder.finish().expect("Should be a valid path")
        }
        style::MarkerShape::Cross => {
            let half_w = marker.size.0 / 2.0;
            let half_h = marker.size.0 / 2.0;
            let mut builder = geom::PathBuilder::new();
            builder.move_to(-half_w, -half_h);
            builder.line_to(half_w, half_h);
            builder.move_to(half_w, -half_h);
            builder.line_to(-half_w, half_h);
            // No close for open shapes
            builder.finish().expect("Should be a valid path")
        }
        style::MarkerShape::Plus => {
            let half_w = marker.size.0 / SQRT2;
            let half_h = marker.size.0 / SQRT2;
            let mut builder = geom::PathBuilder::new();
            builder.move_to(0.0, -half_h);
            builder.line_to(0.0, half_h);
            builder.move_to(-half_w, 0.0);
            builder.line_to(half_w, 0.0);
            // No close for open shapes
            builder.finish().expect("Should be a valid path")
        }
        style::MarkerShape::TriangleUp => {
            let half_w = marker.size.0 / 2.0;
            let half_h = marker.size.0 * SQRT2 / 2.0;
            let mut builder = geom::PathBuilder::new();
            builder.move_to(0.0, -half_h);
            builder.line_to(half_w, half_h);
            builder.line_to(-half_w, half_h);
            builder.close();
            builder.finish().expect("Should be a valid path")
        }
        style::MarkerShape::TriangleDown => {
            let half_w = marker.size.0 / 2.0;
            let half_h = marker.size.0 * SQRT2 / 2.0;
            let mut builder = geom::PathBuilder::new();
            builder.move_to(0.0, half_h);
            builder.line_to(half_w, -half_h);
            builder.line_to(-half_w, -half_h);
            builder.close();
            builder.finish().expect("Should be a valid path")
        }
    }
}