use std::fs;
use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, RgbaImage};

use crate::error::Result;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn clamp01(self) -> Self {
        Self {
            r: self.r.clamp(0.0, 1.0),
            g: self.g.clamp(0.0, 1.0),
            b: self.b.clamp(0.0, 1.0),
        }
    }

    pub fn lerp(self, other: Self, alpha: f32) -> Self {
        let beta = 1.0 - alpha;
        Self {
            r: self.r * beta + other.r * alpha,
            g: self.g * beta + other.g * alpha,
            b: self.b * beta + other.b * alpha,
        }
    }

    pub fn luma(self) -> f32 {
        self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722
    }

    pub fn abs_diff(self, other: Self) -> f32 {
        ((self.r - other.r).abs() + (self.g - other.g).abs() + (self.b - other.b).abs()) / 3.0
    }
}

#[derive(Clone, Debug)]
pub struct ImageFrame {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

impl ImageFrame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::rgb(0.0, 0.0, 0.0); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        self.pixels[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: Color) {
        self.pixels[y * self.width + x] = value;
    }

    pub fn sample_clamped(&self, x: i32, y: i32) -> Color {
        let clamped_x = x.clamp(0, self.width as i32 - 1) as usize;
        let clamped_y = y.clamp(0, self.height as i32 - 1) as usize;
        self.get(clamped_x, clamped_y)
    }

    pub fn sample_bilinear_clamped(&self, x: f32, y: f32) -> Color {
        let x0 = x.floor();
        let y0 = y.floor();
        let x1 = x0 + 1.0;
        let y1 = y0 + 1.0;
        let tx = (x - x0).clamp(0.0, 1.0);
        let ty = (y - y0).clamp(0.0, 1.0);

        let c00 = self.sample_clamped(x0 as i32, y0 as i32);
        let c10 = self.sample_clamped(x1 as i32, y0 as i32);
        let c01 = self.sample_clamped(x0 as i32, y1 as i32);
        let c11 = self.sample_clamped(x1 as i32, y1 as i32);

        let top = c00.lerp(c10, tx);
        let bottom = c01.lerp(c11, tx);
        top.lerp(bottom, ty)
    }

    pub fn to_rgba_image(&self) -> RgbaImage {
        let width = self.width as u32;
        let height = self.height as u32;
        ImageBuffer::from_fn(width, height, |x, y| {
            let color = self.get(x as usize, y as usize).clamp01();
            Rgba([
                (color.r * 255.0).round() as u8,
                (color.g * 255.0).round() as u8,
                (color.b * 255.0).round() as u8,
                255,
            ])
        })
    }

    pub fn encode_png(&self) -> Result<Vec<u8>> {
        let image = DynamicImage::ImageRgba8(self.to_rgba_image());
        let mut cursor = Cursor::new(Vec::new());
        image.write_to(&mut cursor, ImageFormat::Png)?;
        Ok(cursor.into_inner())
    }

    pub fn save_png(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.to_rgba_image().save(path)?;
        Ok(())
    }

    pub fn crop(&self, bbox: BoundingBox) -> Self {
        let mut cropped = ImageFrame::new(bbox.width(), bbox.height());
        for y in 0..bbox.height() {
            for x in 0..bbox.width() {
                cropped.set(x, y, self.get(bbox.min_x + x, bbox.min_y + y));
            }
        }
        cropped
    }
}

#[derive(Clone, Debug)]
pub struct ScalarField {
    width: usize,
    height: usize,
    values: Vec<f32>,
}

impl ScalarField {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.values[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        self.values[y * self.width + x] = value;
    }

    pub fn mean(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f32>() / self.values.len() as f32
    }

    pub fn mean_over_mask(&self, mask: &[bool]) -> f32 {
        let mut sum = 0.0;
        let mut count = 0usize;
        for (value, include) in self.values.iter().zip(mask.iter().copied()) {
            if include {
                sum += *value;
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            sum / count as f32
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl BoundingBox {
    pub fn width(self) -> usize {
        self.max_x - self.min_x + 1
    }

    pub fn height(self) -> usize {
        self.max_y - self.min_y + 1
    }

    pub fn expand(self, width: usize, height: usize, margin: usize) -> Self {
        Self {
            min_x: self.min_x.saturating_sub(margin),
            min_y: self.min_y.saturating_sub(margin),
            max_x: (self.max_x + margin).min(width.saturating_sub(1)),
            max_y: (self.max_y + margin).min(height.saturating_sub(1)),
        }
    }
}

pub fn bounding_box_from_mask(mask: &[bool], width: usize, height: usize) -> Option<BoundingBox> {
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    let mut found = false;

    for y in 0..height {
        for x in 0..width {
            if mask[y * width + x] {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    found.then_some(BoundingBox {
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

pub fn mean_abs_error(frame_a: &ImageFrame, frame_b: &ImageFrame) -> f32 {
    let mut sum = 0.0;
    for (pixel_a, pixel_b) in frame_a.pixels().iter().zip(frame_b.pixels()) {
        sum += pixel_a.abs_diff(*pixel_b);
    }
    sum / frame_a.len() as f32
}

pub fn mean_abs_error_over_mask(frame_a: &ImageFrame, frame_b: &ImageFrame, mask: &[bool]) -> f32 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for ((pixel_a, pixel_b), include) in frame_a
        .pixels()
        .iter()
        .zip(frame_b.pixels())
        .zip(mask.iter().copied())
    {
        if include {
            sum += pixel_a.abs_diff(*pixel_b);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f32
    }
}

pub fn save_scalar_field_png(
    field: &ScalarField,
    path: &Path,
    mapper: impl Fn(f32) -> [u8; 4],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let image = ImageBuffer::from_fn(field.width as u32, field.height as u32, |x, y| {
        let rgba = mapper(field.get(x as usize, y as usize));
        Rgba(rgba)
    });
    image.save(path)?;
    Ok(())
}
