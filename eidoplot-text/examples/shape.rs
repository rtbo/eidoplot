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

    println!("shaping {}", MIXED_TEXT);
    let shape = eidoplot_text::shape_text(MIXED_TEXT, &style, &db).unwrap();
    println!("{:#?}", shape);
}
