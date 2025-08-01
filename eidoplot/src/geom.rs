
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub struct Size {
    pub width: f32,
    pub height: f32,
}

pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h } 
    }
    pub fn new_ps(top_left: Point, size: Size) -> Self {
        Rect::new(top_left.x, top_left.y, size.width, size.height) 
    }
}
