const ENGLISH_TEXT: &str = "Hello, world!
How are you?";

const ARABIC_TEXT: &str = "مرحبا، العالم!
كيف حالك؟";

const MIXED_TEXT_LTR: &str = "Hello, العالم!
How are you?";

const MIXED_TEXT_RTL: &str = "مرحبا، world!
كيف حالك؟";

const ENGLISH_THEN_ARABIC_TEXT: &str = "Hello, world!
كيف حالك؟";

fn main() {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let font = eidoplot_text::Font::default().with_families(vec![
        eidoplot_text::font::Family::Named("Noto Sans".to_string()),
        eidoplot_text::font::Family::Named("DejaVu Sans".to_string()),
        eidoplot_text::font::Family::SansSerif,
    ]);

    let font_size: f32 = 36.0;

    let renders = &[
        (
            ENGLISH_TEXT,
            eidoplot_text::layout::Options {
                anchor: eidoplot_text::layout::Anchor::X,
                hor_align: eidoplot_text::layout::HorAlign::Start,
                hor_justify: false,
                ver_align: eidoplot_text::layout::VerAlign::Top,
            },
            (20.0, 20.0),
        ),
        (
            ARABIC_TEXT,
            eidoplot_text::layout::Options {
                anchor: Default::default(),
                hor_align: eidoplot_text::layout::HorAlign::Start,
                hor_justify: false,
                ver_align: eidoplot_text::layout::VerAlign::Top,
            },
            (580.0, 20.0),
        ),
        (
            MIXED_TEXT_LTR,
            eidoplot_text::layout::Options {
                anchor: eidoplot_text::layout::Anchor::X,
                hor_align: eidoplot_text::layout::HorAlign::Start,
                hor_justify: false,
                ver_align: eidoplot_text::layout::LineVerAlign::Baseline.into(),
            },
            (20.0, 236.0),
        ),
        (
            MIXED_TEXT_RTL,
            eidoplot_text::layout::Options {
                anchor: Default::default(),
                hor_align: eidoplot_text::layout::HorAlign::Start,
                hor_justify: false,
                ver_align: eidoplot_text::layout::LineVerAlign::Hanging.into(),
            },
            (580.0, 236.0),
        ),
        (
            ENGLISH_THEN_ARABIC_TEXT,
            eidoplot_text::layout::Options {
                anchor: eidoplot_text::layout::Anchor::Window(250.0),
                hor_align: eidoplot_text::layout::HorAlign::Start,
                hor_justify: false,
                ver_align: eidoplot_text::layout::VerAlign::Line(
                    1,
                    eidoplot_text::layout::LineVerAlign::Middle,
                ),
            },
            (100.0, 400.0),
        ),
    ];

    let mut pm = tiny_skia::Pixmap::new(600, 500).unwrap();
    let mut pm_mut = pm.as_mut();
    pm_mut.fill(tiny_skia::Color::WHITE);

    for (text, layout_opts, (x, y)) in renders {
        let shape = eidoplot_text::shape2::TextShape::shape_str(text, &font, &db).unwrap();
        let layout = eidoplot_text::layout::TextLayout::from_shape(&shape, font_size, &layout_opts);

        let (tx, ty) = (*x, *y);
        let render_opts = eidoplot_text::render2::Options {
            fill: Some(tiny_skia::Paint::default()),
            outline: None,
            transform: tiny_skia::Transform::from_translate(tx, ty),
            mask: None,
        };

        eidoplot_text::render2::render_text_tiny_skia(&layout, &render_opts, &db, &mut pm_mut);

        draw_layout_bboxes(&layout, (tx, ty), &mut pm_mut);
        draw_anchor_cross(&layout, (tx, ty), &mut pm_mut);
    }

    pm.save_png("out.png").unwrap();
}

fn draw_anchor_cross(
    layout: &eidoplot_text::layout::TextLayout,
    (tx, ty): (f32, f32),
    pm_mut: &mut tiny_skia::PixmapMut,
) {
    let anchor1 = (tx, ty);
    let anchor2 = match layout.options().anchor {
        eidoplot_text::layout::Anchor::X => None,
        eidoplot_text::layout::Anchor::Window(width) => Some((tx + width, ty)),
    };

    let mut pb = tiny_skia_path::PathBuilder::new();
    push_anchor_cross(&mut pb, anchor1);
    if let Some(anchor2) = anchor2 {
        push_anchor_cross(&mut pb, anchor2);
    }
    let path = pb.finish().unwrap();

    let paint = tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 50, 50, 255)),
        ..tiny_skia::Paint::default()
    };
    let stroke = tiny_skia::Stroke::default();
    pm_mut.stroke_path(
        &path,
        &paint,
        &stroke,
        tiny_skia::Transform::identity(),
        None,
    );
}

fn push_anchor_cross(pb: &mut tiny_skia_path::PathBuilder, anchor: (f32, f32)) {
    pb.move_to(anchor.0 - 10.0, anchor.1);
    pb.line_to(anchor.0 + 10.0, anchor.1);
    pb.move_to(anchor.0, anchor.1 - 10.0);
    pb.line_to(anchor.0, anchor.1 + 10.0);
}

fn draw_layout_bboxes(
    layout: &eidoplot_text::layout::TextLayout,
    (tx, ty): (f32, f32),
    pm_mut: &mut tiny_skia::PixmapMut,
) {
    let tr = tiny_skia::Transform::from_translate(tx, ty);
    draw_bbox(
        layout.bbox().transform(&tr),
        tiny_skia::Color::from_rgba8(50, 255, 50, 255),
        2.0,
        false,
        pm_mut,
    );
    for lidx in 0..layout.lines_len() {
        let bbox = layout.line_bbox(lidx).transform(&tr);
        draw_bbox(
            bbox,
            tiny_skia::Color::from_rgba8(50, 50, 255, 255),
            1.0,
            true,
            pm_mut,
        );
    }
}

fn bbox_rect_path(bbox: eidoplot_text::layout::BBox) -> tiny_skia_path::Path {
    let mut pb = tiny_skia_path::PathBuilder::new();
    pb.move_to(bbox.left, bbox.top);
    pb.line_to(bbox.right, bbox.top);
    pb.line_to(bbox.right, bbox.bottom);
    pb.line_to(bbox.left, bbox.bottom);
    pb.line_to(bbox.left, bbox.top);
    pb.finish().unwrap()
}

fn draw_bbox(
    bbox: eidoplot_text::layout::BBox,
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
