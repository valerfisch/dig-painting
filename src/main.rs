use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use std::collections::HashMap;
use std::rc::Rc;

mod generation;

fn render(
    canvas: &mut WindowCanvas,
    image: &generation::artistic::Image,
    brush_textures: &mut HashMap<String, Texture>,
) -> Result<(), String> {
    canvas.set_draw_color(image.color);
    canvas.clear();

    // let (width, height) = canvas.output_size()?;

    for i in &image.strokes {
        let sprite = Rect::from_center(
            i.0.position,
            (i.1.dimensions.0 as f32 * i.0.scale) as u32,
            (i.1.dimensions.1 as f32 * i.0.scale) as u32,
        );

        let texture = brush_textures
            .get_mut(&i.1.texture_path.clone())
            .expect("could not acces texture");

        texture.set_alpha_mod(i.0.opacity);
        texture.set_color_mod(i.0.color.r, i.0.color.g, i.0.color.b);

        canvas.copy_ex(
            &texture,
            None,
            sprite,
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
    let palette = generation::artistic::init_palette(&target);

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

    let texture_creator = Rc::new(canvas.texture_creator());

    let brushes = generation::artistic::init_brushes();
    let mut brush_textures: HashMap<String, Texture> = HashMap::new();

    for brush in &brushes {
        let texture = texture_creator.load_texture(brush.texture_path.clone())?;
        brush_textures.insert(brush.texture_path.to_string(), texture);
    }

    let mut image = generation::artistic::init_image(&target);

    let mut event_pump = sdl_context.event_pump()?;

    let mut delta = f64::MAX;

    let mut i = 0;
    let mut step = 0;
    let mut last_increment = 0;
    let mut lod = 2;

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

        image = image.paint(&brushes, &palette, &target, lod, i);

        // Render
        render(&mut canvas, &image, &mut brush_textures)?;

        // Update
        i += 1;
        // save image
        let buffer: &[u8] = &canvas.read_pixels(
            Rect::from((0, 0, target.dimensions.0, target.dimensions.1)),
            PixelFormatEnum::ABGR8888,
        )?;

        let mut name = format!("./rendered/image-{}.png", i);
        if i < 10 {
            name = format!("./rendered/image-000{}.png", i);
        } else if i < 100 {
            name = format!("./rendered/image-00{}.png", i);
        } else if i < 1000 {
            name = format!("./rendered/image-0{}.png", i);
        }

        let diff = target.compare(buffer);

        if diff >= delta {
            image.strokes.pop();
        }

        if diff < delta && i > 1 {
            last_increment = i;
            delta = diff;
            step += 1;
            // image::save_buffer(&Path::new(&name), buffer, 1920, 1080, ColorType::Rgba8)
            //     .expect("Could not save image");
        }

        if i - last_increment > 15 * 2 ^ i {
            lod += 1;
            last_increment = i;
        }
    }

    Ok(())
}
