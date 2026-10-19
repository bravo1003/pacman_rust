//! Monochrome text rendering that reproduces `TTF_RenderText_Solid` from
//! SDL_ttf 2.24 (FreeType 2.13 with HarfBuzz), pixel for pixel, for the
//! hinting-free pixel fonts the game uses.
//!
//! Glyph outlines are scaled with FreeType's 16.16 fixed-point arithmetic.
//! A pixel is set when its centre is inside the outline; a centre lying
//! exactly on an edge counts as inside for edges to its left, and outside
//! for edges to its right, which is how FreeType resolves the half-pixel
//! edges of the small font. Kerning is not applied: neither font kerns any
//! of the pairs the game can display.

use ttf_parser::{Face, GlyphId, OutlineBuilder};

/// A rendered line of text: `true` marks a set pixel.
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<bool>,
}

pub struct Font {
    face: Face<'static>,
    /// Font units to 26.6 pixels, as FreeType's 16.16 `x_scale`.
    scale: i64,
    ascent: i32,
    height: i32,
}

/// A glyph scaled to 26.6 fixed point, with the pixel box SDL_ttf lays it
/// out by.
struct Glyph {
    edges: Vec<Edge>,
    advance: i32,
    left: i32,
    top: i32,
    width: i32,
    rows: i32,
}

#[derive(Clone, Copy)]
struct Edge {
    from: (i64, i64),
    to: (i64, i64),
}

impl Font {
    pub fn new(data: &'static [u8], pixel_size: i64) -> Result<Self, String> {
        let face = Face::parse(data, 0).map_err(|e| format!("unable to parse font: {e}"))?;
        let scale = div_fix(pixel_size << 6, i64::from(face.units_per_em()));
        // FreeType takes the vertical metrics from the `hhea` table.
        let hhea = face.tables().hhea;
        let ascender = i64::from(hhea.ascender);
        let descender = i64::from(hhea.descender);
        Ok(Self {
            ascent: ceil_px(mul_fix(ascender, scale)),
            height: ceil_px(mul_fix(ascender - descender, scale)),
            face,
            scale,
        })
    }

    pub fn render(&self, text: &str) -> Bitmap {
        let glyphs: Vec<Glyph> = text.chars().map(|c| self.glyph(c)).collect();

        // Lay out like SDL_ttf: a box from the origin down to the font
        // height, grown to fit every glyph's box and the full advance.
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (0, 0, 0, self.height);
        let mut pen = 0;
        let mut origins = Vec::with_capacity(glyphs.len());
        for glyph in &glyphs {
            origins.push(pen);
            let x = pen + glyph.left;
            let y = self.ascent - glyph.top;
            min_x = min_x.min(x);
            max_x = max_x.max(x + glyph.width);
            min_y = min_y.min(y);
            max_y = max_y.max(y + glyph.rows);
            pen += glyph.advance;
        }
        max_x = max_x.max(pen);

        let width = max_x - min_x;
        let height = max_y - min_y;
        let mut pixels = vec![false; (width * height) as usize];
        for (glyph, origin) in glyphs.iter().zip(origins) {
            for row in 0..glyph.rows {
                // Pixel row centres, counted down from the top of the box.
                let center_y = i64::from(glyph.top - row) * 64 - 32;
                let y = self.ascent - glyph.top + row - min_y;
                for col in 0..glyph.width {
                    let center_x = i64::from(glyph.left + col) * 64 + 32;
                    if glyph.covers(center_x, center_y) {
                        let x = origin + glyph.left + col - min_x;
                        pixels[(y * width + x) as usize] = true;
                    }
                }
            }
        }
        Bitmap {
            width: width as u32,
            height: height as u32,
            pixels,
        }
    }

    fn glyph(&self, c: char) -> Glyph {
        let id = self.face.glyph_index(c).unwrap_or(GlyphId(0));
        let mut outline = Outline::default();
        self.face.outline_glyph(id, &mut outline);
        let edges: Vec<Edge> = outline
            .edges
            .iter()
            .map(|edge| Edge {
                from: self.scale_point(edge.from),
                to: self.scale_point(edge.to),
            })
            .collect();

        // Hinting rounds the advance to whole pixels.
        let advance = i64::from(self.face.glyph_hor_advance(id).unwrap_or(0));
        let advance = ((mul_fix(advance, self.scale) + 32) >> 6) as i32;

        let points = || edges.iter().flat_map(|edge| [edge.from, edge.to]);
        let Some(x_min) = points().map(|p| p.0).min() else {
            return Glyph {
                edges,
                advance,
                left: 0,
                top: 0,
                width: 0,
                rows: 0,
            };
        };
        let x_max = points().map(|p| p.0).max().unwrap_or(x_min);
        let y_min = points().map(|p| p.1).min().unwrap_or(0);
        let y_max = points().map(|p| p.1).max().unwrap_or(0);
        let left = floor_px(x_min);
        let top = ceil_px(y_max);
        Glyph {
            advance,
            left,
            top,
            width: ceil_px(x_max) - left,
            rows: top - floor_px(y_min),
            edges,
        }
    }

    fn scale_point(&self, (x, y): (i64, i64)) -> (i64, i64) {
        (mul_fix(x, self.scale), mul_fix(y, self.scale))
    }
}

impl Glyph {
    /// Non-zero winding test at a pixel centre, in 26.6 coordinates.
    fn covers(&self, x: i64, y: i64) -> bool {
        let mut winding = 0;
        for edge in &self.edges {
            let ((x0, y0), (x1, y1)) = (edge.from, edge.to);
            if y0 == y1 || y < y0.min(y1) || y >= y0.max(y1) {
                continue;
            }
            // Only crossings strictly left of the centre count.
            let crossing = x0 as f64 + (y - y0) as f64 * (x1 - x0) as f64 / (y1 - y0) as f64;
            if crossing < x as f64 {
                winding += if y1 > y0 { 1 } else { -1 };
            }
        }
        winding != 0
    }
}

/// Collects outline segments in font units, flattening any curves.
#[derive(Default)]
struct Outline {
    edges: Vec<Edge>,
    start: (i64, i64),
    pen: (f32, f32),
}

impl Outline {
    fn line(&mut self, x: f32, y: f32) {
        let from = (self.pen.0.round() as i64, self.pen.1.round() as i64);
        let to = (x.round() as i64, y.round() as i64);
        if from != to {
            self.edges.push(Edge { from, to });
        }
        self.pen = (x, y);
    }
}

impl OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pen = (x, y);
        self.start = (x.round() as i64, y.round() as i64);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x0, y0) = self.pen;
        for step in 1..=8 {
            let t = step as f32 / 8.0;
            let u = 1.0 - t;
            self.line(
                u * u * x0 + 2.0 * u * t * x1 + t * t * x,
                u * u * y0 + 2.0 * u * t * y1 + t * t * y,
            );
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x0, y0) = self.pen;
        for step in 1..=8 {
            let t = step as f32 / 8.0;
            let u = 1.0 - t;
            self.line(
                u * u * u * x0 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x,
                u * u * u * y0 + 3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t * y,
            );
        }
    }

    fn close(&mut self) {
        let (x, y) = self.start;
        self.line(x as f32, y as f32);
    }
}

/// FreeType's `FT_MulFix`: `a * b / 65536`, rounded half away from zero.
fn mul_fix(a: i64, b: i64) -> i64 {
    let ab = a * b;
    (ab + 0x8000 - i64::from(ab < 0)) >> 16
}

/// FreeType's `FT_DivFix`: `a * 65536 / b`, rounded.
fn div_fix(a: i64, b: i64) -> i64 {
    let sign = a.signum() * b.signum();
    let (a, b) = (a.abs(), b.abs());
    sign * (((a << 16) + (b >> 1)) / b)
}

fn floor_px(value: i64) -> i32 {
    (value >> 6) as i32
}

fn ceil_px(value: i64) -> i32 {
    ((value + 63) >> 6) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::fonts;

    fn rows(bitmap: &Bitmap) -> Vec<String> {
        bitmap
            .pixels
            .chunks(bitmap.width as usize)
            .map(|row| row.iter().map(|&set| if set { '#' } else { '.' }).collect())
            .collect()
    }

    #[test]
    fn large_font_sizes_match_sdl_ttf() {
        let font = Font::new(fonts::EMULOGIC, 24).unwrap();
        for (text, width) in [
            ("Score", 120),
            ("High Score", 240),
            ("ready!", 144),
            ("game  over", 240),
        ] {
            let bitmap = font.render(text);
            assert_eq!((bitmap.width, bitmap.height), (width, 30), "{text:?}");
        }
    }

    #[test]
    fn small_font_half_pixel_edges_match_sdl_ttf() {
        let font = Font::new(fonts::VP_PIXEL, 20).unwrap();
        let three = font.render("3");
        assert_eq!((three.width, three.height), (9, 29));
        let expected = [
            ".#####...",
            ".#####...",
            "......##.",
            "......##.",
            "......##.",
            "......##.",
            "....##...",
            "....##...",
            "......##.",
            "......##.",
            "......##.",
            "......##.",
            "......##.",
            ".#####...",
            ".#####...",
        ];
        assert_eq!(rows(&three)[8..23], expected);
        assert!(
            rows(&three)[..8]
                .iter()
                .chain(&rows(&three)[23..])
                .all(|row| !row.contains('#'))
        );
        assert_eq!(font.render("1600").width, 34);
    }
}
