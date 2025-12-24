use eidoplot_base::geom;
use eidoplot_text::{bundled_font_db, font, line};
use line::LineText;

fn main() {
    let db = bundled_font_db();

    let font = font::Font::default().with_families(vec![
        font::Family::Named("Noto Sans".to_string()),
        font::Family::Named("DejaVu Sans".to_string()),
        font::Family::SansSerif,
    ]);

    let texts = &[
        (
            "Axe des abscisses",
            (line::Align::Start, line::VerAlign::Top),
            (20.0, 20.0),
        ),
        (
            "خط البيانات 124",
            (line::Align::Start, line::VerAlign::Baseline),
            (580.0, 80.0),
        ),
        (
            "Graph title",
            (line::Align::Start, Default::default()),
            (420.0, 236.0),
        ),
    ];

    let mut pm = tiny_skia::Pixmap::new(600, 500).unwrap();
    let mut pm_mut = pm.as_mut();
    pm_mut.fill(tiny_skia::Color::WHITE);

    for (text, align, (x, y)) in texts {
        let (tx, ty) = (*x, *y);
        let render_opts = line::RenderOptions {
            fill: Some(tiny_skia::Paint::default()),
            outline: None,
            transform: tiny_skia::Transform::from_translate(tx, ty),
            mask: None,
        };
        let line = LineText::new(text.to_string(), *align, 32.0, font.clone(), &db).unwrap();
        line::render_line_text(&line, &render_opts, &db, &mut pm_mut);
        draw_line_bbox(&line, (tx, ty), &mut pm_mut);
    }

    pm.save_png("text_line.png").unwrap();
}

fn draw_line_bbox(line: &line::LineText, (tx, ty): (f32, f32), pm_mut: &mut tiny_skia::PixmapMut) {
    let tr = tiny_skia::Transform::from_translate(tx, ty);
    draw_bbox(
        line.bbox().unwrap().transform(&tr),
        tiny_skia::Color::from_rgba8(128, 50, 50, 255),
        2.0,
        false,
        pm_mut,
    );
}

fn draw_bbox(
    rect: geom::Rect,
    color: tiny_skia::Color,
    width: f32,
    dash: bool,
    pm_mut: &mut tiny_skia::PixmapMut,
) {
    let path = rect.to_path();
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
