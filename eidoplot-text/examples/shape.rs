const ENGLISH_TEXT: &str = "Hello, world!
How are you?";

const ARABIC_TEXT: &str = "مرحبا، العالم!
كيف حالك؟";

const MIXED_TEXT_LTR: &str = "Hello, العالم!
How are you?";

const MIXED_TEXT_RTL: &str = "مرحبا، world!
كيف حالك؟";

fn main() {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let style =
        eidoplot_text::style::Font::default().with_family("'Noto Sans','DejaVu Sans',sans-serif".into());
    let font_size: f32 = 36.0;

    let renders = &[
        (
            ENGLISH_TEXT,
            eidoplot_text::TextAlign {
                hor: eidoplot_text::HorAlign::Start,
                ver: eidoplot_text::TextVerAlign::Line(0, eidoplot_text::LineVerAlign::Baseline),
                justify: false,
            },
            (20.0, 36.0),
        ),
        (
            ARABIC_TEXT,
            eidoplot_text::TextAlign {
                hor: eidoplot_text::HorAlign::Start,
                ver: eidoplot_text::TextVerAlign::Line(0, eidoplot_text::LineVerAlign::Baseline),
                justify: false,
            },
            (580.0, 36.0),
        ),
        (
            MIXED_TEXT_LTR,
            eidoplot_text::TextAlign {
                hor: eidoplot_text::HorAlign::Start,
                ver: eidoplot_text::TextVerAlign::Line(0, eidoplot_text::LineVerAlign::Baseline),
                justify: false,
            },
            (20.0, 236.0),
        ),
        (
            MIXED_TEXT_RTL,
            eidoplot_text::TextAlign {
                hor: eidoplot_text::HorAlign::Start,
                ver: eidoplot_text::TextVerAlign::Line(0, eidoplot_text::LineVerAlign::Baseline),
                justify: false,
            },
            (580.0, 236.0),
        ),
    ];

    let mut pxl = tiny_skia::Pixmap::new(600, 300).unwrap();
    let mut pxl_mut = pxl.as_mut();
    pxl_mut.fill(tiny_skia::Color::WHITE);

    for (text, align, (x, y)) in renders {
        let shape = eidoplot_text::shape_text(text, &style, &db).unwrap();

        let width = shape.width(font_size);
        let height = shape.height(font_size);
        println!("size: {} x {}", width, height);

        eidoplot_text::render_text(
            &shape,
            tiny_skia::Transform::from_translate(*x, *y),
            *align,
            font_size,
            &db,
            &mut pxl_mut,
        );
    }

    pxl.save_png("out.png").unwrap();
}
