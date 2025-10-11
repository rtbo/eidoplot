use eidoplot_text::{Font, RichTextBuilder};
use eidoplot_text::{font, rich};
use tiny_skia::Transform;

fn main() {
    let mut db = font::Database::new();
    db.load_system_fonts();

    const FS1: f32 = 36.0;
    const FS2: f32 = 24.0;

    let line1 = "Bode diagram of RLC circuit\n";
    let line2 = "L = 0.1 mH  -  C = 1 µF";
    let text = line1.to_string() + line2;

    let start_rlc = text.find("RLC").unwrap();
    let end_rlc = start_rlc + "RLC".len();

    let start_line2 = line1.len();
    let end_line2 = line1.len() + line2.len();

    let font = Font::default().with_families(vec![
        font::Family::Named("Noto Sans".to_string()),
        font::Family::Named("DejaVu Sans".to_string()),
        font::Family::SansSerif,
    ]);

    let root_props = rich::TextProps::new(FS1).with_font(font);

    let mut builder = RichTextBuilder::new(text, root_props).with_layout(
        rich::Layout::Horizontal(
            rich::Align::Center,
            rich::TypeAlign::Center,
            rich::Direction::LTR,
        ),
    );
    builder.add_span(
        start_rlc,
        end_rlc,
        rich::TextOptProps {
            font_weight: Some(font::Weight::BOLD),
            font_style: Some(font::Style::Italic),
            ..Default::default()
        },
    );
    builder.add_span(
        start_line2,
        end_line2,
        rich::TextOptProps {
            font_size: Some(FS2),
            font_style: Some(font::Style::Italic),
            ..Default::default()
        },
    );

    let text = builder.shape_and_layout(&db).unwrap();
    #[cfg(debug_assertions)]
    text.assert_flat_coverage();

    let mut pm = tiny_skia::Pixmap::new(600, 500).unwrap();
    let mut pm_mut = pm.as_mut();
    pm_mut.fill(tiny_skia::Color::WHITE);

    rich::render_rich_text(
        &text,
        &db,
        Transform::from_translate(300.0, 250.0),
        None,
        &mut pm_mut,
    )
    .unwrap();

    pm.save_png("rich.png").unwrap();
}
