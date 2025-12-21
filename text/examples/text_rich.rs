use eidoplot_text::{self as text, Font, RichTextBuilder, font, rich};
use tiny_skia::Transform;

fn main() {
    let db = text::bundled_font_db();

    const PM_SIZE: (u32, u32) = (600, 500);

    const FS_LARGE: f32 = 36.0;
    const FS_MEDIUM: f32 = 24.0;
    const FS_SMALL: f32 = 16.0;

    let sans_font = Font::default().with_families(vec![
        font::Family::Named("Noto Sans".to_string()),
        font::Family::Named("DejaVu Sans".to_string()),
        font::Family::SansSerif,
    ]);

    let serif_family = vec![
        font::Family::Named("Noto Serif".to_string()),
        font::Family::Named("DejaVu Serif".to_string()),
        font::Family::Serif,
    ];

    let mut pm = tiny_skia::Pixmap::new(PM_SIZE.0, PM_SIZE.1).unwrap();
    let mut pm_mut = pm.as_mut();
    pm_mut.fill(tiny_skia::Color::WHITE);

    // Horizontal english text

    let line1 = "Bode diagram of RLC circuit\n";
    let line2 = "R = 1 \u{03A9}  -  L = 100 \u{03BC}H  -  C = 1 \u{03BC}F";
    let text = line1.to_string() + line2;

    let start_rlc = text.find("RLC").unwrap();
    let end_rlc = start_rlc + "RLC".len();

    let start_line2 = line1.len();
    let end_line2 = line1.len() + line2.len();

    let root_props = rich::TextProps::new(FS_LARGE).with_font(sans_font.clone());

    let mut builder = RichTextBuilder::new(text, root_props).with_layout(rich::Layout::Horizontal(
        rich::Align::Center,
        rich::VerAlign::Center,
        rich::Direction::LTR,
    ));
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
            font_family: Some(serif_family),
            font_size: Some(FS_MEDIUM),
            font_style: Some(font::Style::Italic),
            ..Default::default()
        },
    );

    let text = builder.done(&db, &()).unwrap();
    #[cfg(debug_assertions)]
    text.assert_flat_coverage();

    text::render_rich_text(
        &text,
        &db,
        Transform::from_translate((PM_SIZE.0 / 2) as f32, (PM_SIZE.1 / 2) as f32),
        None,
        &mut pm_mut,
    )
    .unwrap();

    // Vertical chinese text
    let text = "縦書き";

    let serif_cjk_font = Font::default().with_families(vec![
        font::Family::Named("Noto Serif CJK JP".to_string()),
        font::Family::Serif,
    ]);

    let root_props = rich::TextProps::new(FS_LARGE).with_font(serif_cjk_font);
    let builder =
        RichTextBuilder::new(text.to_string(), root_props).with_layout(rich::Layout::Vertical(
            rich::Align::Start,
            rich::HorAlign::Right,
            Default::default(),
            Default::default(),
            Default::default(),
        ));

    let text = builder.done(&db, &()).unwrap();

    text::render_rich_text(
        &text,
        &db,
        Transform::from_translate(PM_SIZE.0 as f32, 0.0),
        None,
        &mut pm_mut,
    )
    .unwrap();

    // Vertical french text
    let text = "Axe des ordonnées";

    let root_props = rich::TextProps::new(FS_SMALL).with_font(sans_font);
    let builder =
        RichTextBuilder::new(text.to_string(), root_props).with_layout(rich::Layout::Vertical(
            rich::Align::End,
            rich::HorAlign::Left,
            Default::default(),
            Default::default(),
            Default::default(),
        ));

    let text = builder.done(&db, &()).unwrap();

    text::render_rich_text(
        &text,
        &db,
        Transform::from_translate(0.0, PM_SIZE.1 as f32),
        None,
        &mut pm_mut,
    )
    .unwrap();

    pm.save_png("text_rich.png").unwrap();
}
