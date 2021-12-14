use colors_transform::{Color, Rgb};

struct Target {
  image: image::RgbaImage,
  dimensions: (u32, u32), // hsv: &[u8],
}

pub fn open_target_image() {
  let target = image::open("assets/targets/dino.png").expect("could not open target");

  let target = target.to_rgba8();
  let dimensions = target.dimensions();

  // let hsv = Vec::new();

  for image::Rgba(p) in target.pixels() {
    let [r, g, b, a] = &p;

    println!("r: {}, g: {}, b: {}, a: {}", &r, &g, &b, &a);
    let hsl = Rgb::from(*r as f32, *g as f32, *b as f32).to_hsl();

    println!(
      "h: {}, s: {}, l: {}",
      hsl.get_hue(),
      hsl.get_saturation(),
      hsl.get_lightness()
    )
  }

  let target = Target {
    image: target,
    dimensions,
  };
}
