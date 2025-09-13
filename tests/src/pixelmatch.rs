//! pixelmatch algorithm, taken from https://github.com/dfrankland/pixelmatch-rs
//! itself adapted from JS pixelmatch from https://github.com/mapbox/pixelmatch
//! and adapted here for tiny-skia pixmap.
//! Because it is only used in tests, the errors are reported through panics only.

// pixelmatch-rs from https://github.com/dfrankland/pixelmatch-rs
// is released under the MIT license with the following copyright:
// Copyright (c) 2021, Dylan Frankland

// JS pixelmatch from https://github.com/mapbox/pixelmatch
// is released under the ISC license with the following copyright:
// Copyright (c) 2025, Mapbox

use core::f64;

use tiny_skia::{ColorU8, Pixmap, PixmapRef};

pub struct Options {
    /// matching threshold (0 to 1); smaller is more sensitive
    pub threshold: f64,
    /// whether to skip anti-aliasing detection
    pub include_aa: bool,
    /// opacity of original image in diff output
    pub alpha: f64,
    /// color of anti-aliased pixels in diff output
    pub aa_color: ColorU8,
    /// color of different pixels in diff output
    pub diff_color: ColorU8,
    /// whether to detect dark on light differences between img1 and img2 and set an alternative color to differentiate between the two
    pub diff_color_alt: Option<ColorU8>,
    /// draw the diff over a transparent background (a mask)
    pub diff_mask: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            threshold: 0.1,
            include_aa: true,
            alpha: 0.1,
            aa_color: ColorU8::from_rgba(255, 255, 0, 255),
            diff_color: ColorU8::from_rgba(255, 0, 0, 255),
            diff_color_alt: None,
            diff_mask: true,
        }
    }
}

trait PixmapExt {
    fn demultiplied_pixel(&self, x: u32, y: u32) -> ColorU8;
}

trait PixmapMutExt {
    fn set_demultiplied_pixel(&mut self, x: u32, y: u32, color: ColorU8);
}

impl PixmapExt for PixmapRef<'_> {
    fn demultiplied_pixel(&self, x: u32, y: u32) -> ColorU8 {
        let idx = (self.width() * y + x) as usize;
        self.pixels()[idx].demultiply()
    }
}

impl PixmapMutExt for Pixmap {
    fn set_demultiplied_pixel(&mut self, x: u32, y: u32, color: ColorU8) {
        let idx = (self.width() * y + x) as usize;
        self.pixels_mut()[idx] = color.premultiply();
    }
}

pub fn pixelmatch(
    img1: PixmapRef,
    img2: PixmapRef,
    options: Option<Options>,
) -> (Option<Pixmap>, usize) {
    if img1.width() != img2.width() || img1.height() != img1.height() {
        panic!("Image sizes do not match.");
    }

    let (width, height) = (img1.width(), img1.height());

    let options = options.unwrap_or_default();

    // fast path if identical
    if img1 == img2 {
        let mut img_out = Pixmap::new(width, height).unwrap();
        if !options.diff_mask {
            for (px_src, px_dst) in img1.pixels().iter().zip(img_out.pixels_mut()) {
                *px_dst = gray_pixel(px_src.demultiply(), options.alpha).premultiply();
            }
        }
        return (Some(img_out), 0);
    }

    // maximum acceptable square distance between two colors;
    // 35215 is the maximum possible value for the YIQ difference metric
    let max_delta = 35215_f64 * options.threshold * options.threshold;
    let mut diff: usize = 0;

    let mut img_out = None;

    for x in 0..width {
        for y in 0..height {
            let pixel1 = img1.demultiplied_pixel(x, y);
            let pixel2 = img2.demultiplied_pixel(x, y);

            let delta = color_delta(pixel1, pixel2, false);

            if delta.abs() > max_delta {
                // check it's a real rendering difference or just anti-aliasing
                if !options.include_aa
                    && (antialiased(img1, x, y, img2) || antialiased(img2, x, y, img1))
                {
                    // one of the pixels is anti-aliasing; draw as yellow and do not count as difference
                    // note that we do not include such pixels in a mask

                    if !options.diff_mask {
                        if img_out.is_none() {
                            img_out = Some(Pixmap::new(width, height).unwrap());
                        }
                        img_out
                            .as_mut()
                            .unwrap()
                            .set_demultiplied_pixel(x, y, options.aa_color);
                    }
                } else {
                    // found substantial difference not caused by anti-aliasing; draw it as such
                    let color = if delta < 0.0 {
                        options.diff_color_alt.unwrap_or(options.diff_color)
                    } else {
                        options.diff_color
                    };
                    if img_out.is_none() {
                        img_out = Some(Pixmap::new(width, height).unwrap());
                    }
                    img_out
                        .as_mut()
                        .unwrap()
                        .set_demultiplied_pixel(x, y, color);
                    diff += 1;
                }
            }
        }
    }

    (img_out, diff)
}

fn gray_pixel(pixel: ColorU8, alpha: f64) -> ColorU8 {
    let val = blend(
        rgb2y(pixel.red(), pixel.green(), pixel.blue()),
        (alpha * pixel.alpha() as f64) / 255.0,
    ) as u8;

    ColorU8::from_rgba(val, val, val, val)
}

// check if a pixel is likely a part of anti-aliasing;
// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
fn antialiased(img1: PixmapRef, x: u32, y: u32, img2: PixmapRef) -> bool {
    let width = img1.width();
    let height = img1.height();

    let mut zeroes: u8 = if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
        1
    } else {
        0
    };

    let mut min = 0.0;
    let mut max = 0.0;

    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;

    let center_rgba = img1.pixel(x, y).unwrap().demultiply();

    for adjacent_x in (if x > 0 { x - 1 } else { x })..=(if x < width - 1 { x + 1 } else { x }) {
        for adjacent_y in (if y > 0 { y - 1 } else { y })..=(if y < height - 1 { y + 1 } else { y })
        {
            if adjacent_x == x && adjacent_y == y {
                continue;
            }

            // brightness delta between the center pixel and adjacent one
            let rgba = img1.pixel(adjacent_x, adjacent_y).unwrap().demultiply();
            let delta = color_delta(center_rgba, rgba, true);

            // count the number of equal, darker and brighter adjacent pixels
            if delta == 0.0 {
                zeroes += 1;

                // if found more than 2 equal siblings, it's definitely not anti-aliasing
                if zeroes > 2 {
                    return false;
                }

                continue;
            }

            // remember the darkest pixel
            if delta < min {
                min = delta;
                min_x = adjacent_x;
                min_y = adjacent_y;

                continue;
            }

            // remember the brightest pixel
            if delta > max {
                max = delta;
                max_x = adjacent_x;
                max_y = adjacent_y;
            }
        }
    }

    // if there are no both darker and brighter pixels among siblings, it's not anti-aliasing
    if min == 0.0 || max == 0.0 {
        return false;
    }

    // if either the darkest or the brightest pixel has 3+ equal siblings in both images
    // (definitely not anti-aliased), this pixel is anti-aliased
    (has_many_siblings(img1, min_x, min_y, width, height)
        && has_many_siblings(img2, min_x, min_y, width, height))
        || (has_many_siblings(img1, max_x, max_y, width, height)
            && has_many_siblings(img2, max_x, max_y, width, height))
}

// check if a pixel has 3+ adjacent pixels of the same color.
fn has_many_siblings(img: PixmapRef, x: u32, y: u32, width: u32, height: u32) -> bool {
    let mut zeroes: u8 = if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
        1
    } else {
        0
    };

    let center_rgba = img.pixel(x, y).unwrap().demultiply();

    for adjacent_x in (if x > 0 { x - 1 } else { x })..=(if x < width - 1 { x + 1 } else { x }) {
        for adjacent_y in (if y > 0 { y - 1 } else { y })..=(if y < height - 1 { y + 1 } else { y })
        {
            if adjacent_x == x && adjacent_y == y {
                continue;
            }

            let rgba = img.pixel(adjacent_x, adjacent_y).unwrap().demultiply();

            if center_rgba == rgba {
                zeroes += 1;
            }

            if zeroes > 2 {
                return true;
            }
        }
    }

    false
}

// calculate color difference according to the paper "Measuring perceived color difference
// using YIQ NTSC transmission color space in mobile applications" by Y. Kotsarenko and F. Ramos
fn color_delta(rgba1: ColorU8, rgba2: ColorU8, y_only: bool) -> f64 {
    let mut r1 = rgba1.red() as f64;
    let mut g1 = rgba1.green() as f64;
    let mut b1 = rgba1.blue() as f64;
    let mut a1 = rgba1.alpha() as f64;

    let mut r2 = rgba2.red() as f64;
    let mut g2 = rgba2.green() as f64;
    let mut b2 = rgba2.blue() as f64;
    let mut a2 = rgba2.alpha() as f64;

    if (a1 - a2).abs() < f64::EPSILON
        && (r1 - r2).abs() < f64::EPSILON
        && (g1 - g2).abs() < f64::EPSILON
        && (b1 - b2).abs() < f64::EPSILON
    {
        return 0.0;
    }

    if a1 < 255.0 {
        a1 /= 255.0;
        r1 = blend(r1, a1);
        g1 = blend(g1, a1);
        b1 = blend(b1, a1);
    }

    if a2 < 255.0 {
        a2 /= 255.0;
        r2 = blend(r2, a2);
        g2 = blend(g2, a2);
        b2 = blend(b2, a2);
    }

    let y1 = rgb2y(r1, g1, b1);
    let y2 = rgb2y(r2, g2, b2);
    let y = y1 - y2;

    // brightness difference only
    if y_only {
        return y;
    }

    let i = rgb2i(r1, g1, b1) - rgb2i(r2, g2, b2);
    let q = rgb2q(r1, g1, b1) - rgb2q(r2, g2, b2);

    let delta = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;

    // encode whether the pixel lightens or darkens in the sign
    if y1 > y2 { -delta } else { delta }
}

// blend semi-transparent color with white
fn blend<T: Into<f64>>(c: T, a: T) -> f64 {
    255.0 + (c.into() - 255.0) * a.into()
}

fn rgb2y<T: Into<f64>>(r: T, g: T, b: T) -> f64 {
    r.into() * 0.29889531 + g.into() * 0.58662247 + b.into() * 0.11448223
}
fn rgb2i<T: Into<f64>>(r: T, g: T, b: T) -> f64 {
    r.into() * 0.59597799 - g.into() * 0.27417610 - b.into() * 0.32180189
}
fn rgb2q<T: Into<f64>>(r: T, g: T, b: T) -> f64 {
    r.into() * 0.21147017 - g.into() * 0.52261711 + b.into() * 0.31114694
}
