use chrono::{Datelike, Timelike, Utc};
use image::ColorType;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::libc::int64_t;
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
    stop: usize,
) -> Result<(), String> {
    canvas.set_draw_color(image.color);
    canvas.clear();

    // let (width, height) = canvas.output_size()?;

    let mut counter = 0;

    for i in &image.strokes {
        let sprite = Rect::new(
            i.0.position.x,
            i.0.position.y,
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
            i.0.rotation,
            None,
            false,
            false,
        )?;

        if counter >= stop {
            break;
        }

        counter += 1;
    }

    canvas.present();

    Ok(())
}

fn main() -> Result<(), String> {
    let target = generation::comparison::open_target_image("assets/targets/valerie_1.jpg");
    let palette = generation::artistic::init_palette("assets/targets/valerie_1.jpg");

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
    let mut lod = 0;
    let mut placed = 0;
    let mut next = false;
    let mut run = false;
    let mut storing = false;

    let mut backup_canvas_buffer = canvas.read_pixels(
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
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    i += 1;
                    next = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    run = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    storing = true;
                }
                _ => {}
            }
        }

        if next || run {
            if run {
                i += 1;
            }

            image = image.paint(&brushes, &palette, &target, lod);

            // Render
            render(&mut canvas, &image, &mut brush_textures, i)?;

            // Update
            // i += 1;

            // save image
            let stroke = image.strokes[image.strokes.len() - 1].clone();

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
                    // step += 1;
                }
            }

            if (placed % (lod.max(1) * 100) == 0 && i == last_increment  && storing) || placed == 50000 {
                let now = Utc::now();

                let name = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second(),);
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

            if placed > 10 && i - last_increment > (2_i32.pow(lod as u32)) as usize && lod < 6 {
                lod += 1;
                last_increment = i;
            }

        println!(
            "{}, {}, {}, {}, {}",
            lod,
            step.max(i - last_increment),
            (2_i32.pow(lod as u32)) as usize,
            delta - diff > 0.0,
            placed
        );

        if step < i - last_increment {
            step = i - last_increment
        }

        if placed >= 50000 {
                break;
            }
            next = false;
        }
    }

    Ok(())
}
