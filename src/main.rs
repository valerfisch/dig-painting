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

fn render(canvas: &mut WindowCanvas, image: &generation::artistic::Image) -> Result<(), String> {
    canvas.set_draw_color(image.color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    for i in &image.strokes {
        let sprite = Rect::new(0, 0, i.1.dimensions.0, i.1.dimensions.1);

        let screen_position = i.0.position;
        let screen_rect = Rect::from_center(
            screen_position,
            (sprite.width() as f32 * i.0.scale) as u32,
            (sprite.height() as f32 * i.0.scale) as u32,
        );

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.load_texture(i.1.texture_path.clone())?;

        texture.set_alpha_mod(i.0.opacity);
        texture.set_color_mod(i.0.color.r, i.0.color.g, i.0.color.b);

        canvas.copy_ex(
            &texture,
            None,
            screen_rect,
            i.0.rotation * 360.0,
            None,
            false,
            false,
        )?;
    }

    canvas.present();

    Ok(())
}

fn main() -> Result<(), String> {
    let target = generation::comparison::open_target_image();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem
        .window("dig painting", target.dimensions.0, target.dimensions.1)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    let brushes = generation::artistic::init_brushes();

    let mut image = generation::artistic::init_image(&target);

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
                    break 'running;
                }
                _ => {}
            }
        }

        image = image.paint(&brushes);

        // Render
        render(&mut canvas, &image)?;

        // Update
        i += 1;
        // save image
        let buffer: &[u8] =
            &canvas.read_pixels(Rect::from((0, 0, 1920, 1080)), PixelFormatEnum::ABGR8888)?;

        let name = format!("./rendered/image-{}.png", i);

        image::save_buffer(&Path::new(&name), buffer, 1920, 1080, ColorType::Rgba8)
            .expect("Could not save image");
    }

    /*
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

    image.strokes.push((stroke, brush.clone()));
    image.strokes.push((stroke2, brush.clone()));

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.load_texture(brush.texture_path.clone())?;

    }
    */

    Ok(())
}
