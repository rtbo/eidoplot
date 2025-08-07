use crate::style::defaults;


#[derive(Debug, Clone)]
pub struct Family(pub String);

impl Family {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for Family {
    fn default() -> Self {
        Family(defaults::FONT_FAMILY.into())
    }
}

impl<S> From<S> for Family
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Family(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    family: Family,
    size: f32,
}

impl Font {
    pub fn new(family: Family, size: f32) -> Self {
        Font { family, size }
    }

    pub fn family(&self) -> &Family {
        &self.family
    }

    pub fn size(&self) -> f32 {
        self.size
    }
}

impl Default for Font {
    fn default() -> Self {
        Font {
            family: Family::default(),
            size: 24.0,
        }
    }
}

impl From<Family> for Font {
    fn from(value: Family) -> Self {
        Font {
            family: value,
            ..Font::default()
        }
    }
}

impl From<(Family, f32)> for Font {
    fn from(value: (Family, f32)) -> Self {
        Font {
            family: value.0,
            size: value.1,
        }
    }
}

impl From<f32> for Font {
    fn from(value: f32) -> Self {
        Font {
            size: value,
            ..Font::default()
        }
    }
}
