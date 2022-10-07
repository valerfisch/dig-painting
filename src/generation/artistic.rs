use crate::generation::comparison;
use rand::Rng;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::fmt;
use std::fs;
use std::io;
use std::rc::Rc;

use super::comparison::Target;

const LOD: [std::ops::Range<f32>; 6] = [
    (0.5..1.0),
    (0.3..0.5),
    (0.2..0.3),
    (0.1..0.2),
    (0.05..0.1),
    (0.02..0.05),
];
const MIN_MAGNITUDE: [f32; 6] = [f32::MIN, 0.1, 0.3, 0.5, 0.6, 0.7];

pub struct Palette {
    colors: Vec<Color>,
}

#[derive(Clone)]
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
#[derive(Clone)]
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
    pub dimensions: (u32, u32),
    pub color: Color,
    pub strokes: Vec<(Stroke, Rc<Brush>)>,
}

impl Image {
    pub fn paint(
        mut self,
        brushes: &Vec<Brush>,
        palette: &Palette,
        tar: &Target,
        lod: usize,
    ) -> Image {
        let mut rng = rand::thread_rng();

        let mut position = Point::new(
            rng.gen_range(0..self.dimensions.0).try_into().unwrap(),
            rng.gen_range(0..self.dimensions.1).try_into().unwrap(),
        );

        let mut magnitude =
            tar.magnitudes[(position.y as u32 * self.dimensions.0 + position.x as u32) as usize];

        while magnitude < MIN_MAGNITUDE[lod.min(5)] {
            position = Point::new(
                rng.gen_range(0..self.dimensions.0).try_into().unwrap(),
                rng.gen_range(0..self.dimensions.1).try_into().unwrap(),
            );

            magnitude = tar.magnitudes
                [(position.y as u32 * self.dimensions.0 + position.x as u32) as usize];
        }

        let rotation =
            tar.angles[(position.y as u32 * self.dimensions.0 + position.x as u32) as usize];
            
        let idx = rng.gen_range(0..brushes.len());
        
        let stroke = Stroke {
            position,
            rotation: (rotation) as f64,
            // RGB
            scale: rng.gen_range(LOD[lod.min(5)].clone()),
            color: palette.colors[rng.gen_range(0..palette.colors.len())].clone(),
            opacity: rng.gen_range(55..180),
        };

        let brush = Rc::new(Brush {
            texture_path: brushes[idx].texture_path.clone(),
            dimensions: (
                brushes[idx].dimensions.0.clone(),
                brushes[idx].dimensions.1.clone(),
            ),
        });
        self.strokes.push((stroke, brush.clone()));
        self
    }
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

pub fn init_image(target: &comparison::Target) -> Image {
    Image {
        strokes: Vec::new(),
        color: Color::from((127, 127, 127)),
        dimensions: (target.dimensions.0, target.dimensions.1),
    }
}

pub fn init_palette(path: &str) -> Palette {
    let palett_src = image::open(path).expect("could not open target").to_rgba8();

    let mut colors: Vec<Color> = Vec::new();

    let row_width =  50;
    let col_height = 50;

    let row_count = palett_src.dimensions().1 / col_height;
    let col_count = palett_src.dimensions().0 / row_width;

    for row in 0..row_count {
        for col in 0..col_count {
            let mut c = Color::RGB(127, 127, 127);
            for y in 0..col_height {
                for x in 0..row_width {
                    let x_pos = col * row_width + x;
                    let y_pos = row * col_height + y;
                    let image::Rgba(p) = palett_src.get_pixel(x_pos as u32, y_pos as u32);
                    let [mut r, mut g, mut b, _] = &p;

                    r = ((c.r as u16 + r as u16) / 2) as u8;
                    g = ((c.g as u16 + g as u16) / 2) as u8;
                    b = ((c.b as u16 + b as u16) / 2) as u8;

                    c = Color::RGB(r, g, b)
                }
            }
            colors.push(c)
        }
    }

    return Palette { colors };
}
