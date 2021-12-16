use colors_transform::{Color, Rgb};
use image::{GenericImageView, LumaA};
use rayon::prelude::*;

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
    pub fn compare(&self, buf: &[u8]) -> f64 {
        let size: usize = (self.dimensions.0 * self.dimensions.1) as usize;

        let delta: Vec<(f32, f32, f32)> = (0..size)
            .into_par_iter()
            .map(|i| -> (f32, f32, f32) {
                let r = buf[(i * 4) as usize];
                let g = buf[((i * 4) + 1) as usize];
                let b = buf[((i * 4) + 2) as usize];

                let hsl = Rgb::from(r as f32, g as f32, b as f32).to_hsl();
                return (
                    ((self.hsls[i as usize].0 - hsl.get_hue()) / 4.0).abs(),
                    (self.hsls[i as usize].1 - hsl.get_saturation()).abs(),
                    (self.hsls[i as usize].2 - hsl.get_lightness()).abs(),
                );
            })
            .collect();

        let delta: Vec<f64> = (0..size)
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
    let img = image::open("assets/targets/mona_lisa.jpg").expect("could not open target");

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
