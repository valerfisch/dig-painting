use image::{save_buffer, ColorType};
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::path::Path;
use std::rc::Rc;
use std::time::Duration; // Import `fmt`

mod generation;

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &mut Texture,
    image: &generation::artistic::Image,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    for i in &image.strokes {
        let sprite = Rect::new(0, 0, i.1.dimensions.0, i.1.dimensions.1);

        let screen_position = i.0.position + Point::new(width as i32 / 4, height as i32 / 2);
        let screen_rect = Rect::from_center(
            screen_position,
            (sprite.width() as f32 * i.0.scale / 16.0) as u32,
            (sprite.height() as f32 * i.0.scale / 16.0) as u32,
        );

        texture.set_alpha_mod(i.0.opacity);
        texture.set_color_mod(i.0.color.r, i.0.color.g, i.0.color.b);

        // canvas.copy(texture, sprite, screen_rect)?;
        canvas.copy_ex(texture, None, screen_rect, i.0.rotation, None, false, false)?;
    }

    canvas.present();

    Ok(())
}

fn main() -> Result<(), String> {
    generation::artistic::init_brushes();
    generation::comparison::open_target_image();

    /*

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("game tutorial", 1920, 1080)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let brush = Rc::new(generation::artistic::Brush {
        dimensions: (2267, 906),
        texture_path: "assets/stroke_2.png".to_string(),
    });

    let stroke = generation::artistic::Stroke {
        position: Point::new(100, 0),
        rotation: 1.0,
        // RGB
        scale: 2.5,
        color: Color::RGB(255, 180, 162),
        opacity: 70,
    };

    let stroke2 = generation::artistic::Stroke {
        position: Point::new(100, 100),
        rotation: 45.0,
        // RGB
        scale: 2.5,
        color: Color::RGB(181, 131, 141),
        opacity: 150,
    };

    let mut image = generation::artistic::Image {
        strokes: Vec::new(),
    };

    image.strokes.push((stroke, brush.clone()));
    image.strokes.push((stroke2, brush.clone()));

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.load_texture(brush.texture_path.clone())?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    let buffer: &[u8] = &canvas
                        .read_pixels(Rect::from((0, 0, 1920, 1080)), PixelFormatEnum::ABGR8888)?;
                    image::save_buffer(
                        &Path::new("image.png"),
                        buffer,
                        1920,
                        1080,
                        ColorType::Rgba8,
                    )
                    .expect("Could not save image");
                    break 'running;
                }
                _ => {}
            }
        }

        // Update
        i = (i + 1) % 255;

        // Render
        render(&mut canvas, Color::RGB(80, 80, 80), &mut texture, &image)?;

        image.strokes[1].0.rotation += 1.0;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    */

    Ok(())
}
