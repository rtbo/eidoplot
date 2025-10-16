use eidoplot_text::{font, line};
use line::Line;

use eidoplot_text as text;

fn main() {
    let mut db = font::Database::new();
    db.load_system_fonts();

    let font = font::Font::default().with_families(vec![
        font::Family::Named("Noto Sans".to_string()),
        font::Family::Named("DejaVu Sans".to_string()),
        font::Family::SansSerif,
    ]);

    let texts = &[
        (
            "Axe des abscisses",
            line::Options {
                align: line::Align::Start,
                baseline: line::Baseline::Top,
                font: font.clone(),
            },
            (20.0, 20.0),
        ),
        (
            "خط البيانات 123",
            line::Options {
                align: line::Align::Start,
                baseline: line::Baseline::Baseline,
                font: font.clone(),
            },
            (580.0, 80.0),
        ),
        (
            "Graph title",
            line::Options {
                align: line::Align::Start,
                baseline: Default::default(),
                font: font.clone(),
            },
            (420.0, 236.0),
        ),
    ];

    let mut pm = tiny_skia::Pixmap::new(600, 500).unwrap();
    let mut pm_mut = pm.as_mut();
    pm_mut.fill(tiny_skia::Color::WHITE);

    for (text, line_opts, (x, y)) in texts {
        let (tx, ty) = (*x, *y);
        let render_opts = text::render::Options {
            fill: Some(tiny_skia::Paint::default()),
            outline: None,
            transform: tiny_skia::Transform::from_translate(tx, ty),
            mask: None,
        };
        let line = Line::new(text.to_string(), 32.0, line_opts, &db).unwrap();
        text::render::render_line(&line, &render_opts, &db, &mut pm_mut);
        draw_line_bbox(&line, (tx, ty), &mut pm_mut);
    }

    pm.save_png("text_line.png").unwrap();
}

fn draw_line_bbox(line: &line::Line, (tx, ty): (f32, f32), pm_mut: &mut tiny_skia::PixmapMut) {
    let tr = tiny_skia::Transform::from_translate(tx, ty);
    draw_bbox(
        line.bbox().transform(&tr),
        tiny_skia::Color::from_rgba8(128, 50, 50, 255),
        2.0,
        false,
        pm_mut,
    );
    // for lidx in 0..layout.lines_len() {
    //     let bbox = layout.line_bbox(lidx).transform(&tr);
    //     draw_bbox(
    //         bbox,
    //         tiny_skia::Color::from_rgba8(50, 50, 255, 255),
    //         1.0,
    //         true,
    //         pm_mut,
    //     );
    // }
}

fn bbox_rect_path(bbox: text::BBox) -> tiny_skia_path::Path {
    let mut pb = tiny_skia_path::PathBuilder::new();
    pb.move_to(bbox.left, bbox.top);
    pb.line_to(bbox.right, bbox.top);
    pb.line_to(bbox.right, bbox.bottom);
    pb.line_to(bbox.left, bbox.bottom);
    pb.line_to(bbox.left, bbox.top);
    pb.finish().unwrap()
}

fn draw_bbox(
    bbox: text::BBox,
    color: tiny_skia::Color,
    width: f32,
    dash: bool,
    pm_mut: &mut tiny_skia::PixmapMut,
) {
    let path = bbox_rect_path(bbox);
    draw_path_stroke(&path, color, width, dash, pm_mut);
}

fn draw_path_stroke(
    path: &tiny_skia::Path,
    color: tiny_skia::Color,
    width: f32,
    dash: bool,
    pm_mut: &mut tiny_skia::PixmapMut,
) {
    let paint = tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(color),
        ..tiny_skia::Paint::default()
    };
    let dash = if dash {
        tiny_skia::StrokeDash::new(vec![width * 2.0, width * 2.0], 0.0)
    } else {
        None
    };
    let stroke = tiny_skia::Stroke {
        width,
        dash,
        ..Default::default()
    };
    let transform = tiny_skia::Transform::identity();
    let mask = None;
    pm_mut.stroke_path(path, &paint, &stroke, transform, mask);
}
