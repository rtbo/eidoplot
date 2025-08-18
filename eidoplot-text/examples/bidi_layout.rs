
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
            eidoplot_text::LayoutOptions {
                hor_align: eidoplot_text::HorAlign::Start,
                hor_anchor: eidoplot_text::HorAnchor::X(20.0),
                hor_justify: false,
                ver_align: eidoplot_text::TextVerAlign::Top,
                ver_anchor: eidoplot_text::VerAnchor(20.0), 
            },
            (0.0, 0.0),
        ),
        (
            ARABIC_TEXT,
            eidoplot_text::LayoutOptions {
                hor_align: eidoplot_text::HorAlign::Start,
                hor_anchor: Default::default(),
                hor_justify: false,
                ver_align: eidoplot_text::TextVerAlign::Top,
                ver_anchor: Default::default(), 
            },
            (580.0, 20.0)
        ),
        (
            MIXED_TEXT_LTR,
            eidoplot_text::LayoutOptions {
                hor_align: eidoplot_text::HorAlign::Start,
                hor_anchor: eidoplot_text::HorAnchor::X(20.0),
                hor_justify: false,
                ver_align: eidoplot_text::TextVerAlign::Line(0, eidoplot_text::LineVerAlign::Baseline),
                ver_anchor: Default::default(), 
            },
            (0.0, 236.0),
        ),
        (
            MIXED_TEXT_RTL,
            eidoplot_text::LayoutOptions {
                hor_align: eidoplot_text::HorAlign::Start,
                hor_anchor: Default::default(),
                hor_justify: false,
                ver_align: eidoplot_text::TextVerAlign::Line(0, eidoplot_text::LineVerAlign::Hanging),
                ver_anchor: Default::default(), 
            },
            (580.0, 236.0),
        ),
        (
            ENGLISH_THEN_ARABIC_TEXT,
            eidoplot_text::LayoutOptions {
                hor_align: eidoplot_text::HorAlign::Start,
                hor_anchor: eidoplot_text::HorAnchor::Window {
                    x_left: 100.0,
                    x_right: 350.0,
                },
                hor_justify: false,
                ver_align: eidoplot_text::TextVerAlign::Line(1, eidoplot_text::LineVerAlign::Middle),
                ver_anchor: eidoplot_text::VerAnchor(400.0), 
            },
            (0.0, 0.0),
        ),
    ];

    let mut pxl = tiny_skia::Pixmap::new(600, 500).unwrap();
    let mut pxl_mut = pxl.as_mut();
    pxl_mut.fill(tiny_skia::Color::WHITE);


    for (text, layout_opts, (x, y)) in renders {
        let shape = eidoplot_text::shape_text(text, &font, &db).unwrap();

        let render_opts = eidoplot_text::RenderOptions {
            fill: Some(tiny_skia::Paint::default()),
            outline: None,
            transform: tiny_skia::Transform::from_translate(*x, *y),
            mask: None,
        };

        eidoplot_text::render_text(
            &shape,
            font_size,
            layout_opts,
            &render_opts,
            &db,
            &mut pxl_mut,
        );

        let y = *y + layout_opts.ver_anchor.0;

        let p1x = match layout_opts.hor_anchor {
            eidoplot_text::HorAnchor::X(xx) => x + xx,
            eidoplot_text::HorAnchor::Window { x_left, .. } => x + x_left,
        };

        let p2 = match layout_opts.hor_anchor {
            eidoplot_text::HorAnchor::X(..) => None,
            eidoplot_text::HorAnchor::Window { x_right, .. } => Some(x + x_right),
        };

        let mut pb = tiny_skia_path::PathBuilder::new();
        pb.move_to(p1x-10.0, y);
        pb.line_to(p1x+10.0, y);
        pb.move_to(p1x, y - 10.0);
        pb.line_to(p1x, y + 10.0);

        if let Some(p2x) = p2 {
            pb.move_to(p2x - 10.0, y);
            pb.line_to(p2x + 10.0, y);
            pb.move_to(p2x,  y - 10.0);
            pb.line_to(p2x , y + 10.0);
        }
        let path = pb.finish().unwrap();

        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(255, 50, 50, 255)),
            ..tiny_skia::Paint::default()
        };
        let stroke = tiny_skia::Stroke::default();
        let transform = tiny_skia::Transform::identity();
        let mask = None;
        pxl_mut.stroke_path(&path, &paint, &stroke, transform, mask);
    }

    pxl.save_png("out.png").unwrap();
}
