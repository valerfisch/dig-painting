use chrono::{Datelike, Timelike, Utc};
use image::ColorType;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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

    let now = Utc::now();

    let folder_name = format!(
        "./rendered/{}-{:02}-{:02}_{:02}:{:02}:{:02}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    );

    fs::create_dir(folder_name.clone());

    let mut i = 0;
    let mut step = 0;
    let mut last_increment = 0;
    let mut lod = 2;
    let mut placed = 0;

    let mut backup_canvas_buffer =  canvas.read_pixels(
        Rect::from((0, 0, target.dimensions.0, target.dimensions.1)),
        PixelFormatEnum::ABGR8888,
    )?;

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
        let stroke = image.strokes[image.strokes.len()-1].clone();

        let buffer = canvas.read_pixels(
            Rect::from((0, 0, target.dimensions.0, target.dimensions.1)),
            PixelFormatEnum::ABGR8888,
        )?;

        let diff = target.compare(&buffer, stroke.clone());
        let delta = target.compare(&backup_canvas_buffer, stroke.clone());

        if diff >= delta {
            image.strokes.pop();
        } else {
            placed = placed + 1;

            backup_canvas_buffer.copy_from_slice(&buffer);

            if i > 1 {
                last_increment = i;
                step += 1;
            }
        }



        if placed % 25 == 0 && i == last_increment {
            let now = Utc::now();

            let name = format!(
                "{:02}:{:02}:{:02}",
                now.hour(),
                now.minute(),
                now.second(),
            );
            let name = format!("{}/{}.png", folder_name, name);

            image::save_buffer(
                &Path::new(&name),
                &buffer,
                target.dimensions.0,
                target.dimensions.1,
                ColorType::Rgba8,
            )
            .expect("Could not save image");
        }

        if i > 1000 && i - last_increment > (15 * 2_i32.pow(lod as u32)) as usize {
            lod += 1;
            last_increment = i;
        }

        println!(
            "{}, {}, {}, {}, {}, {}",
            i,
            lod,
            delta - diff > 0.0,
            i - last_increment,
            (150 * 2_i32.pow(lod as u32)) as usize,
            placed
        );

        if placed >= 5000 {
            break;
        }
    }

    Ok(())
}
