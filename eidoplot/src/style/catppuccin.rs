use crate::style;
use crate::style::ColorU8;

#[derive(Debug, Clone, Copy)]
pub struct Latte;

#[derive(Debug, Clone, Copy)]
pub struct Frappe;

#[derive(Debug, Clone, Copy)]
pub struct Macchiato;

#[derive(Debug, Clone, Copy)]
pub struct Mocha;

trait Flavors {
    fn rosewater() -> ColorU8;
    fn flamingo() -> ColorU8;
    fn pink() -> ColorU8;
    fn mauve() -> ColorU8;
    fn red() -> ColorU8;
    fn maroon() -> ColorU8;
    fn peach() -> ColorU8;
    fn yellow() -> ColorU8;
    fn green() -> ColorU8;
    fn teal() -> ColorU8;
    fn sky() -> ColorU8;
    fn sapphire() -> ColorU8;
    fn blue() -> ColorU8;
    fn lavender() -> ColorU8;
    fn text() -> ColorU8;
    fn _subtext1() -> ColorU8;
    fn _subtext0() -> ColorU8;
    fn overlay2() -> ColorU8;
    fn _overlay1() -> ColorU8;
    fn _overlay0() -> ColorU8;
    fn surface2() -> ColorU8;
    fn _surface1() -> ColorU8;
    fn surface0() -> ColorU8;
    fn base() -> ColorU8;
    fn _mantle() -> ColorU8;
    fn _crust() -> ColorU8;
}

trait IsDark {
    fn is_dark() -> bool;
}

impl<F> style::theme::Theme for F 
where F: Flavors + IsDark + style::series::Palette,
{
    type Palette = Self;

    fn palette(&self) -> &Self::Palette {
        self
    }

    fn is_dark(&self) -> bool {
        F::is_dark()
    }

    fn background(&self) -> ColorU8 {
        F::base()
    }

    fn foreground(&self) -> ColorU8 {
        F::text()
    }

    fn grid(&self) -> ColorU8 {
        F::surface2()
    }

    fn legend_fill(&self) -> ColorU8 {
        F::surface0()
    }

    fn legend_border(&self) -> ColorU8 {
        F::overlay2()
    }
}

impl<F> style::series::Palette for F 
where F: Flavors 
{
    fn len(&self) -> usize {
        14
    }

    fn get(&self, color: style::series::IndexColor) -> ColorU8 {
        match color.0 % 14 {
            0 => F::blue(),
            1 => F::peach(),
            2 => F::green(),
            3 => F::red(),
            4 => F::mauve(),
            5 => F::maroon(),
            6 => F::flamingo(),
            7 => F::pink(),
            8 => F::lavender(),
            9 => F::teal(),
            10 => F::sky(),
            11 => F::yellow(),
            12 => F::sapphire(),
            13 => F::rosewater(),
            _ => unreachable!(),
        }
    }
}

impl IsDark for Latte {
    fn is_dark() -> bool {
        false
    }
}

/// The standard eidoplot color palette (10 colors)
pub const STANDARD: &[ColorU8] = &[
    ColorU8::from_html(b"#1f77b4"), // blue
    ColorU8::from_html(b"#ff7f0e"), // orange
    ColorU8::from_html(b"#2ca02c"), // green
    ColorU8::from_html(b"#d62728"), // red
    ColorU8::from_html(b"#9467bd"), // purple
    ColorU8::from_html(b"#8c564b"), // brown
    ColorU8::from_html(b"#e377c2"), // pink
    ColorU8::from_html(b"#7f7f7f"), // gray
    ColorU8::from_html(b"#bcbd22"), // olive
    ColorU8::from_html(b"#17becf"), // cyan
];

impl Flavors for Latte {
    fn rosewater() -> ColorU8 {
        ColorU8::from_html(b"#dc8a78")
    }
    fn flamingo() -> ColorU8 {
        ColorU8::from_html(b"#dd7878")
    }
    fn pink() -> ColorU8 {
        ColorU8::from_html(b"#ea76cb")
    }
    fn mauve() -> ColorU8 {
        ColorU8::from_html(b"#8839ef")
    }
    fn red() -> ColorU8 {
        ColorU8::from_html(b"#d20f39")
    }
    fn maroon() -> ColorU8 {
        ColorU8::from_html(b"#e64553")
    }
    fn peach() -> ColorU8 {
        ColorU8::from_html(b"#fe640b")
    }
    fn yellow() -> ColorU8 {
        ColorU8::from_html(b"#df8e1d")
    }
    fn green() -> ColorU8 {
        ColorU8::from_html(b"#40a02b")
    }
    fn teal() -> ColorU8 {
        ColorU8::from_html(b"#179299")
    }
    fn sky() -> ColorU8 {
        ColorU8::from_html(b"#04a5e5")
    }
    fn sapphire() -> ColorU8 {
        ColorU8::from_html(b"#209fb5")
    }
    fn blue() -> ColorU8 {
        ColorU8::from_html(b"#1e66f5")
    }
    fn lavender() -> ColorU8 {
        ColorU8::from_html(b"#7287fd")
    }
    fn text() -> ColorU8 {
        ColorU8::from_html(b"#4c4f69")
    }
    fn _subtext1() -> ColorU8 {
        ColorU8::from_html(b"#5c5f77")
    }
    fn _subtext0() -> ColorU8 {
        ColorU8::from_html(b"#6c6f85")
    }
    fn overlay2() -> ColorU8 {
        ColorU8::from_html(b"#7c7f93")
    }
    fn _overlay1() -> ColorU8 {
        ColorU8::from_html(b"#9ca0b0")
    }
    fn _overlay0() -> ColorU8 {
        ColorU8::from_html(b"#c6c8d1")
    }
    fn surface2() -> ColorU8 {
        ColorU8::from_html(b"#dfdfe0")
    }
    fn _surface1() -> ColorU8 {
        ColorU8::from_html(b"#e8e8e8")
    }
    fn surface0() -> ColorU8 {
        ColorU8::from_html(b"#f5f5f5")
    }
    fn base() -> ColorU8 {
        ColorU8::from_html(b"#eff1f5")
    }
    fn _mantle() -> ColorU8 {
        ColorU8::from_html(b"#e6e9ef")
    }
    fn _crust() -> ColorU8 {
        ColorU8::from_html(b"#dce0e8")
    }
}

impl IsDark for Frappe {
    fn is_dark() -> bool {
        true
    }
}

impl Flavors for Frappe {
    fn rosewater() -> ColorU8 {
        ColorU8::from_html(b"#f2d5cf")
    }
    fn flamingo() -> ColorU8 {
        ColorU8::from_html(b"#eebebe")
    }
    fn pink() -> ColorU8 {
        ColorU8::from_html(b"#f4b8e4")
    }
    fn mauve() -> ColorU8 {
        ColorU8::from_html(b"#ca9ee6")
    }
    fn red() -> ColorU8 {
        ColorU8::from_html(b"#e78284")
    }
    fn maroon() -> ColorU8 {
        ColorU8::from_html(b"#ea999c")
    }
    fn peach() -> ColorU8 {
        ColorU8::from_html(b"#ef9f76")
    }
    fn yellow() -> ColorU8 {
        ColorU8::from_html(b"#e5c890")
    }
    fn green() -> ColorU8 {
        ColorU8::from_html(b"#a6d189")
    }
    fn teal() -> ColorU8 {
        ColorU8::from_html(b"#81c8be")
    }
    fn sky() -> ColorU8 {
        ColorU8::from_html(b"#99d1db")
    }
    fn sapphire() -> ColorU8 {
        ColorU8::from_html(b"#85c1dc")
    }
    fn blue() -> ColorU8 {
        ColorU8::from_html(b"#8caaee")
    }
    fn lavender() -> ColorU8 {
        ColorU8::from_html(b"#babbf1")
    }
    fn text() -> ColorU8 {
        ColorU8::from_html(b"#c6d0f5")
    }
    fn _subtext1() -> ColorU8 {
        ColorU8::from_html(b"#b5bfe2")
    }
    fn _subtext0() -> ColorU8 {
        ColorU8::from_html(b"#a5adce")
    }
    fn overlay2() -> ColorU8 {
        ColorU8::from_html(b"#949cbb")
    }
    fn _overlay1() -> ColorU8 {
        ColorU8::from_html(b"#838ba7")
    }
    fn _overlay0() -> ColorU8 {
        ColorU8::from_html(b"#737994")
    }
    fn surface2() -> ColorU8 {
        ColorU8::from_html(b"#626880")
    }
    fn _surface1() -> ColorU8 {
        ColorU8::from_html(b"#51576d")
    }
    fn surface0() -> ColorU8 {
        ColorU8::from_html(b"#414559")
    }
    fn base() -> ColorU8 {
        ColorU8::from_html(b"#303446")
    }
    fn _mantle() -> ColorU8 {
        ColorU8::from_html(b"#292c36")
    }
    fn _crust() -> ColorU8 {
        ColorU8::from_html(b"#232634")
    }
}

impl IsDark for Macchiato {
    fn is_dark() -> bool {
        true
    }
}

impl Flavors for Macchiato {
    fn rosewater() -> ColorU8 {
        ColorU8::from_html(b"#f4dbd6")
    }
    fn flamingo() -> ColorU8 {
        ColorU8::from_html(b"#f0c6c6")
    }
    fn pink() -> ColorU8 {
        ColorU8::from_html(b"#f5bde6")
    }
    fn mauve() -> ColorU8 {
        ColorU8::from_html(b"#c6a0f6")
    }
    fn red() -> ColorU8 {
        ColorU8::from_html(b"#ed8796")
    }
    fn maroon() -> ColorU8 {
        ColorU8::from_html(b"#ee99a0")
    }
    fn peach() -> ColorU8 {
        ColorU8::from_html(b"#f5a97f")
    }
    fn yellow() -> ColorU8 {
        ColorU8::from_html(b"#eed49f")
    }
    fn green() -> ColorU8 {
        ColorU8::from_html(b"#a6da95")
    }
    fn teal() -> ColorU8 {
        ColorU8::from_html(b"#8bd5ca")
    }
    fn sky() -> ColorU8 {
        ColorU8::from_html(b"#91d7e3")
    }
    fn sapphire() -> ColorU8 {
        ColorU8::from_html(b"#7dc4e4")
    }
    fn blue() -> ColorU8 {
        ColorU8::from_html(b"#8aadf4")
    }
    fn lavender() -> ColorU8 {
        ColorU8::from_html(b"#b7bdf8")
    }
    fn text() -> ColorU8 {
        ColorU8::from_html(b"#cad3f5")
    }
    fn _subtext1() -> ColorU8 {
        ColorU8::from_html(b"#b8c0e0")
    }
    fn _subtext0() -> ColorU8 {
        ColorU8::from_html(b"#a5adcb")
    }
    fn overlay2() -> ColorU8 {
        ColorU8::from_html(b"#939ab7")
    }
    fn _overlay1() -> ColorU8 {
        ColorU8::from_html(b"#8087a2")
    }
    fn _overlay0() -> ColorU8 {
        ColorU8::from_html(b"#6e738d")
    }
    fn surface2() -> ColorU8 {
        ColorU8::from_html(b"#5b6078")
    }
    fn _surface1() -> ColorU8 {
        ColorU8::from_html(b"#494d64") 
    }
    fn surface0() -> ColorU8 {
        ColorU8::from_html(b"#363a4f")
    }
    fn base() -> ColorU8 {
        ColorU8::from_html(b"#24273a")
    }
    fn _mantle() -> ColorU8 {
        ColorU8::from_html(b"#1e2030")
    }
    fn _crust() -> ColorU8 {
        ColorU8::from_html(b"#181926")
    }
}

impl IsDark for Mocha {
    fn is_dark() -> bool {
        true
    }
}

impl Flavors for Mocha {
    fn rosewater() -> ColorU8 {
        ColorU8::from_html(b"#f5e0dc")
    }
    fn flamingo() -> ColorU8 {
        ColorU8::from_html(b"#f2cdcd")
    }
    fn pink() -> ColorU8 {
        ColorU8::from_html(b"#f5c2e7")
    }
    fn mauve() -> ColorU8 {
        ColorU8::from_html(b"#cba6f7")
    }
    fn red() -> ColorU8 {
        ColorU8::from_html(b"#f38ba8")
    }
    fn maroon() -> ColorU8 {
        ColorU8::from_html(b"#eba0ac")
    }
    fn peach() -> ColorU8 {
        ColorU8::from_html(b"#fab387")
    }
    fn yellow() -> ColorU8 {
        ColorU8::from_html(b"#f9e2af")
    }
    fn green() -> ColorU8 {
        ColorU8::from_html(b"#a6e3a1")
    }
    fn teal() -> ColorU8 {
        ColorU8::from_html(b"#94e2d5")
    }
    fn sky() -> ColorU8 {
        ColorU8::from_html(b"#89dceb")
    }
    fn sapphire() -> ColorU8 {
        ColorU8::from_html(b"#74c7ec")
    }
    fn blue() -> ColorU8 {
        ColorU8::from_html(b"#89b4fa")
    }
    fn lavender() -> ColorU8 {
        ColorU8::from_html(b"#b4befe")
    }
    fn text() -> ColorU8 {
        ColorU8::from_html(b"#cdd6f4")
    }
    fn _subtext1() -> ColorU8 {
        ColorU8::from_html(b"#bac2de")
    }
    fn _subtext0() -> ColorU8 {
        ColorU8::from_html(b"#a6adc8")
    }
    fn overlay2() -> ColorU8 {
        ColorU8::from_html(b"#9399b2")
    }
    fn _overlay1() -> ColorU8 {
        ColorU8::from_html(b"#7f849c")
    }
    fn _overlay0() -> ColorU8 {
        ColorU8::from_html(b"#6c7086")
    }
    fn surface2() -> ColorU8 {
        ColorU8::from_html(b"#585b70")
    }
    fn _surface1() -> ColorU8 {
        ColorU8::from_html(b"#45475a")
    }
    fn surface0() -> ColorU8 {
        ColorU8::from_html(b"#313244")
    }
    fn base() -> ColorU8 {
        ColorU8::from_html(b"#1e1e2e")
    }
    fn _mantle() -> ColorU8 {
        ColorU8::from_html(b"#181825")
    }
    fn _crust() -> ColorU8 {
        ColorU8::from_html(b"#11111b")
    }
}

