use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::fmt;
use std::fs;
use std::io;
use std::rc::Rc;

#[derive(Debug)]
pub struct Brush {
  pub texture_path: String,
  pub dimensions: (u32, u32),
}

// Similarly, implement `Display` for `Point2D`
impl fmt::Display for Brush {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // Customize so only `x` and `y` are denoted.
    write!(
      f,
      "texture_path: {}, dimensions.x: {}, dimensions.y: {}",
      self.texture_path, self.dimensions.0, self.dimensions.1,
    )
  }
}

#[derive(Debug)]
pub struct Stroke {
  // top-left corner
  pub position: Point,
  pub rotation: f64,
  // RGB
  pub scale: f32,
  pub color: Color,
  pub opacity: u8,
}

// Similarly, implement `Display` for `Point2D`
impl fmt::Display for Stroke {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // Customize so only `x` and `y` are denoted.
    write!(
      f,
      "position.x: {}, posistion.y: {}, rotation: {}, scale: {}, color: {} {} {}, opacity: {}",
      self.position.x(),
      self.position.y(),
      self.rotation,
      self.scale,
      self.color.r,
      self.color.g,
      self.color.b,
      self.opacity,
    )
  }
}

pub struct Image {
  pub strokes: Vec<(Stroke, Rc<Brush>)>,
}

pub fn init_brushes() -> Vec<Brush> {
  let mut entries = fs::read_dir("./assets/brushes")
    .expect("could not read directory")
    .map(|res| res.map(|e| e.path()))
    .collect::<Result<Vec<_>, io::Error>>()
    .expect("could not collect entrys in directory");

  // The order in which `read_dir` returns entries is not guaranteed. If reproducible
  // ordering is required the entries should be explicitly sorted.

  entries.sort();

  let mut brushes = Vec::new();

  for entry in entries {
    let path = entry
      .to_str()
      .as_deref()
      .unwrap_or("default string")
      .to_string();

    let image = image::open(&path).expect("could not open image");
    let image = image.to_rgba8();

    let brush = Brush {
      texture_path: path.clone(),
      dimensions: image.dimensions(),
    };

    println!("{}", brush);
    brushes.push(brush);
  }

  brushes
}
