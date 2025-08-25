use super::{ColorU8};

#[derive(Debug, Clone, Copy)]
pub struct Color(pub usize);

pub trait Palette {
    fn len(&self) -> usize;
    fn get(&self, color: Color) -> ColorU8;
}

#[derive(Debug, Clone, Copy)]
pub struct Standard;

impl Palette for Standard {
    fn len(&self) -> usize {
        10
    }   

    fn get(&self, color: Color) -> ColorU8 {
        let colors = [
            ColorU8::from_rgb_f32(0.121, 0.466, 0.705), // blue
            ColorU8::from_rgb_f32(1.0, 0.498, 0.054),   // orange
            ColorU8::from_rgb_f32(0.172, 0.627, 0.172), // green
            ColorU8::from_rgb_f32(0.839, 0.153, 0.157), // red
            ColorU8::from_rgb_f32(0.580, 0.404, 0.741), // purple
            ColorU8::from_rgb_f32(0.549, 0.337, 0.294), // brown
            ColorU8::from_rgb_f32(0.890, 0.467, 0.761), // pink
            ColorU8::from_rgb_f32(0.498, 0.498, 0.498), // gray
            ColorU8::from_rgb_f32(0.737, 0.741, 0.133), // olive
            ColorU8::from_rgb_f32(0.090, 0.745, 0.811), // cyan
        ];
        colors[color.0 % colors.len()]
    }
}
