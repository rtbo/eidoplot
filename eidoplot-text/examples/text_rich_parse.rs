use eidoplot_color as color;
use eidoplot_text::{Font, font, rich};
use tiny_skia::Transform;
fn main() {
    let mut db = font::Database::new();
    db.load_system_fonts();

    const PM_SIZE: (u32, u32) = (600, 500);

    let sans_font = Font::default().with_families(vec![
        font::Family::Named("Noto Sans".to_string()),
        font::Family::Named("DejaVu Sans".to_string()),
        font::Family::SansSerif,
    ]);

    let mut pm = tiny_skia::Pixmap::new(PM_SIZE.0, PM_SIZE.1).unwrap();
    let mut pm_mut = pm.as_mut();
    pm_mut.fill(tiny_skia::Color::WHITE);
    let fmt = concat!(
        "Bode diagram of [bold]RLC[/bold] filter\n",
        "[size=32]with [color=teal]cutoff frequency[/color] at [italic]1.5 kHz[/italic].[/size]\n",
        "[font='Noto Serif','DejaVu Serif','Times';italic;size=24]R = 1 Ω  -  L = 100 μH  -  C = 1 μF[/font;italic]"
    );
    let rich_text = rich::parse_rich_text(fmt)
        .unwrap()
        .into_builder(
            rich::TextProps::new(36.0)
                .with_font(sans_font)
                .with_fill(Some(color::BLACK)),
        )
        .with_layout(rich::Layout::Horizontal(
            rich::Align::Center,
            rich::VerAlign::Center,
            Default::default(),
        ))
        .done(&db, &())
        .unwrap();

    rich::render_rich_text(
        &rich_text,
        &db,
        Transform::from_translate(PM_SIZE.0 as f32 / 2.0, PM_SIZE.1 as f32 / 2.0),
        None,
        &mut pm_mut,
    )
    .unwrap();

    pm.save_png("text_rich_parse.png").unwrap();
}
