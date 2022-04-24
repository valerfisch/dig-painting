use crate::generation::artistic::Brush;
use crate::generation::artistic::Stroke;
use colors_transform::{Color, Rgb};
use image::{GenericImageView, LumaA};
use rayon::prelude::*;
use std::rc::Rc;

enum SobelType {
    Horizontal,
    Vertical,
}

pub struct Target {
    pub image: image::RgbaImage,
    pub dimensions: (u32, u32),
    pub hsls: Vec<(f32, f32, f32)>,
    //pub edges: edge_detection::Detection,
    pub magnitudes: Vec<f32>,
    pub angles: Vec<f32>,
}

impl Target {
    pub fn compare(&self, buf: &[u8], stroke: (Stroke, Rc<Brush>)) -> f64 {

        let center_position = ((stroke.1.dimensions.0 / 2) as f64, (stroke.1.dimensions.1 / 2) as f64 );
        let angle = stroke.0.rotation;
        let height = stroke.1.dimensions.1 as f64;
        let width = stroke.1.dimensions.0 as f64;

        let top_right = (
            center_position.0 + ((width / 2.0) * angle.cos()) - ((height / 2.0) * angle.sin()),
            center_position.1 + ((width / 2.0) * angle.sin()) + ((height / 2.0) * angle.cos())
        );

        let top_left = (
            center_position.0 - ((width / 2.0) * angle.cos()) - ((height / 2.0) * angle.sin()),
            center_position.1 - ((width / 2.0) * angle.sin()) + ((height / 2.0) * angle.cos())
        );

        let bottom_left = (
            center_position.0 - ((width / 2.0) * angle.cos()) + ((height / 2.0) * angle.sin()),
            center_position.1 - ((width / 2.0) * angle.sin()) - ((height / 2.0) * angle.cos())
        );

        let bottom_right = (
            center_position.0 + ((width / 2.0) * angle.cos()) + ((height / 2.0) * angle.sin()),
            center_position.1 + ((width / 2.0) * angle.sin()) - ((height / 2.0) * angle.cos())
        );
        
        let x_min = top_left.0.min(top_right.0).min(bottom_left.0).min(bottom_right.0);
        let y_min = top_left.1.min(top_right.1).min(bottom_left.1).min(bottom_right.1);
        let x_max = top_left.0.max(top_right.0).max(bottom_left.0).max(bottom_right.0);
        let y_max = top_left.1.max(top_right.1).max(bottom_left.1).max(bottom_right.1);

        let mut stroke_size = (((x_max - x_min) as f32 * stroke.0.scale) as u32, ((y_max - y_min) as f32 * stroke.0.scale) as u32);

        let x_offset = stroke.0.position.x as u32;
        let y_offset = stroke.0.position.y as u32;

        if x_offset + stroke_size.0 > self.dimensions.0 - 1
        {
            stroke_size.0 = self.dimensions.0 - 1 - x_offset
        }

        if y_offset + stroke_size.1 > self.dimensions.1 - 1
        {
            stroke_size.1 = 
                self.dimensions.1 - 1 - y_offset
        }

        let size: u32 = stroke_size.0 * stroke_size.1;
        let delta: Vec<(f32, f32, f32)> = (0..size)
            .into_par_iter()
            .map(|i| -> (f32, f32, f32) {
                let y_min = (y_offset + (i / stroke_size.0)).min(self.dimensions.1 - 1);
                let x_min = (x_offset + i % stroke_size.0).min(self.dimensions.0 - 1);

                let offset = y_min * self.dimensions.0 + x_min;
                let r = buf[(offset * 4_u32) as usize];
                let g = buf[((offset * 4_u32) + 1) as usize];
                let b = buf[((offset * 4_u32) + 2) as usize];
                let hsl = Rgb::from(r as f32, g as f32, b as f32).to_hsl();
                return (
                    ((self.hsls[offset as usize].0 - hsl.get_hue()) / 4.0).abs(),
                    (self.hsls[offset as usize].1 - hsl.get_saturation()).abs(),
                    (self.hsls[offset as usize].2 - hsl.get_lightness()).abs(),
                );
            })
            .collect();

        let delta: Vec<f64> = (0..size as usize)
            .into_par_iter()
            .map(|i| -> f64 {
                return delta[i].0 as f64 + delta[i].1 as f64 + delta[i].2 as f64;
            })
            .collect();

        let sum: f64 = delta.iter().sum();

        sum
    }
}

pub fn open_target_image() -> Target {
    let img = image::open("assets/targets/portrait.jpg").expect("could not open target");

    let img_luma = img.clone().to_luma8();
    let img_rgba = img.to_rgba8();
    let dimensions = img_rgba.dimensions();
    let size: u64 = (&dimensions.0 * &dimensions.1) as u64;

    let horizontal_sobel = sobel_filter(&img_luma, SobelType::Horizontal);
    let vertical_sobel = sobel_filter(&img_luma, SobelType::Vertical);

    let mut magnitudes: Vec<f32> = Vec::new();
    let mut angles: Vec<f32> = Vec::new();

    for y in 0..dimensions.1 {
        for x in 0..dimensions.0 {
            let gx: f32 = horizontal_sobel.get_pixel(x, y).0[0] as f32 / 255.0;
            let gy: f32 = vertical_sobel.get_pixel(x, y).0[0] as f32 / 255.0;

            let radius: f32 = (gx * gx + gy * gy).sqrt();
            let phi: f32 = (gy).atan2(gx);

            magnitudes.push(radius.powf(0.5));
            angles.push(phi * std::f32::consts::PI / 2.0);
        }
    }

    let mut hsls = vec![(0.0 as f32, 0.0 as f32, 0.0 as f32); (size) as usize];

    hsls = (0..size)
        .into_par_iter()
        .map(|i| {
            let x = i % dimensions.0 as u64;
            let y = i / dimensions.0 as u64;
            let image::Rgba(p) = img.get_pixel(x as u32, y as u32);
            let [r, g, b, _] = &p;

            let hsl = Rgb::from(*r as f32, *g as f32, *b as f32).to_hsl();
            return (hsl.get_hue(), hsl.get_saturation(), hsl.get_lightness());
        })
        .collect();

    return Target {
        image: img_rgba,
        dimensions,
        hsls,
        magnitudes,
        angles,
    };
}

fn sobel_filter(
    image: &image::GrayImage,
    sobel_type: SobelType,
) -> image::ImageBuffer<image::Luma<i16>, std::vec::Vec<i16>> {
    let (width, height) = image.dimensions();
    let kernel: [((i16, i16), i16); 6] = match sobel_type {
        SobelType::Horizontal => [
            ((-1, -1), -1),
            ((1, -1), -1),
            ((-1, 0), 2),
            ((1, 0), -2),
            ((-1, 1), 1),
            ((1, 1), -1),
        ],
        SobelType::Vertical => [
            ((-1, -1), 1),
            ((0, -1), 2),
            ((1, -1), 1),
            ((-1, 1), -1),
            ((0, 1), -2),
            ((1, 1), -1),
        ],
    };

    let mut raw: Vec<i16> = Vec::with_capacity((width * height) as usize);

    unsafe {
        raw.set_len(raw.capacity());
    }

    let iter_rows = raw.par_chunks_mut(width as usize).enumerate();
    iter_rows.for_each(move |(y, row_vec): (usize, &mut [i16])| {
        for (x, val) in row_vec.iter_mut().enumerate() {
            *val = kernel
                .iter()
                .map(|(pos, mult)| {
                    let kx = ((x as i16 + pos.0).max(0) as u32).min(width - 1);
                    let ky = ((y as i16 + pos.1).max(0) as u32).min(height - 1);
                    let kp = unsafe { &image.unsafe_get_pixel(kx, ky) };
                    (kp[0] as i16) * mult
                })
                .sum::<i16>();
        }
    });

    image::ImageBuffer::from_raw(width, height, raw).unwrap()
}
