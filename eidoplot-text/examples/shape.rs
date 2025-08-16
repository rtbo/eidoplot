// const WESTERN_TEXT: &str = "Hello, world!
// How are you?";

// const ARABIC_TEXT: &str = "مرحبا، العالم!
// كيف حالك؟";

const MIXED_TEXT: &str = "Hello, العالم!
كيف some intermixed حالك؟";

fn main() {
    let mut db = eidoplot::font::bundled_db();
    db.load_system_fonts();

    let style = eidoplot_text::style::Font::default();

    let shape = eidoplot_text::shape_text(MIXED_TEXT, &style, &db).unwrap();

    let font_size: f32 = 36.0;
    let width = shape.width(font_size);
    let height = shape.height(font_size);
    println!("size: {} x {}", width, height);

    let mut pxl = tiny_skia::Pixmap::new(500, 100).unwrap();
    let mut pxl_mut = pxl.as_mut();
    pxl_mut.fill(tiny_skia::Color::WHITE);

    let align = eidoplot_text::Align(
        eidoplot_text::HorAlign::Center,
        eidoplot_text::VerAlign::Middle,
    );

    eidoplot_text::render_line(
        &shape.lines()[0],
        tiny_skia::Transform::from_translate(250.0, 50.0),
        align,
        font_size,
        None,
        &db,
        &mut pxl_mut,
    );

    pxl.save_png("out.png").unwrap();
}
