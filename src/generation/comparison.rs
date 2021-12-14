use colors_transform::{Color, Rgb};
use rayon::prelude::*;

pub struct Target {
    pub image: image::RgbaImage,
    pub dimensions: (u32, u32),
    pub hsls: Vec<(f32, f32, f32)>,
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
