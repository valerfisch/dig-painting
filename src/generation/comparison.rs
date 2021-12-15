use colors_transform::{Color, Rgb};
use rayon::prelude::*;

pub struct Target {
    pub image: image::RgbaImage,
    pub dimensions: (u32, u32),
    pub hsls: Vec<(f32, f32, f32)>,
}

impl Target {
    pub fn compare(&self, buf: &[u8]) -> f32 {
        let size: u64 = (self.dimensions.0 * self.dimensions.1) as u64;

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

        let delta: Vec<f32> = (0..size)
            .into_par_iter()
            .map(|i| -> f32 {
                return delta[i as usize].0 + delta[i as usize].1 + delta[i as usize].2;
            })
            .collect();

        let sum: f32 = delta.iter().sum();

        sum
    }
}

pub fn open_target_image() -> Target {
    let img = image::open("assets/targets/dino.png").expect("could not open target");

    let img = img.to_rgba8();
    let dimensions = img.dimensions();
    let size: u64 = (&dimensions.0 * &dimensions.1) as u64;

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
        image: img,
        dimensions,
        hsls,
    };
}

fn hsl_calculate_row(
    y: u32,
    width: u32,
    img: &image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
) -> Vec<(f32, f32, f32)> {
    println!("calculating row {}", y);

    let mut hsls = vec![(0.0 as f32, 0.0 as f32, 0.0 as f32); width as usize];

    for x in 0..width {
        let image::Rgba(p) = img.get_pixel(x, y);
        let [r, g, b, _] = &p;

        let hsl = Rgb::from(*r as f32, *g as f32, *b as f32).to_hsl();

        hsls[x as usize] = (hsl.get_hue(), hsl.get_saturation(), hsl.get_lightness());
    }

    hsls
}
