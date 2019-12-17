extern crate image as im;
extern crate piston_window;
extern crate log;
use std::time::Instant;
use std::env;

use self::piston_window::*;

use crate::chip8::Chip8;
const CELL_PIXEL_SIDE: u32 = 20;
const X_SIZE: u32 = CELL_PIXEL_SIDE * 64;
const Y_SIZE: u32 = CELL_PIXEL_SIDE * 32;
const UPS: u32 = 4000;

pub mod chip8;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("missing rom!");
        return;
    }

    let mut window: PistonWindow = WindowSettings::new("Chip-8", [X_SIZE, Y_SIZE])
        .resizable(false)
        .exit_on_esc(true)
        .graphics_api(OpenGL::V3_2)
        .fullscreen(false)
        .build()
        .unwrap();

    window.events = Events::new(EventSettings {
        max_fps: 4000,
        ups: UPS as u64,
        ups_reset: 2,
        swap_buffers: true,
        lazy: false,
        bench_mode: false,
    });

    let mut canvas = im::ImageBuffer::new(X_SIZE as u32, Y_SIZE as u32);
    let mut ctx = window.create_texture_context();

    let mut texture = Texture::from_image(&mut ctx, &canvas, &TextureSettings::new()).unwrap();

    let rom = read_file(&args[1]);

    let mut machine = Chip8::new();

    machine.init();
    machine.load_rom(0x200, &rom);
    machine.set_pc(0x200);

    let mut keyboard = [false; 16];

    while let Some(e) = window.next() {
        let _now = Instant::now();
        if let Some(btn) = e.press_args() {
            match btn {
                Button::Keyboard(Key::D1) => keyboard[0x0] = true,
                Button::Keyboard(Key::D2) => keyboard[0x1] = true,
                Button::Keyboard(Key::D3) => keyboard[0x2] = true,
                Button::Keyboard(Key::D4) => keyboard[0x3] = true,
                Button::Keyboard(Key::Q) => keyboard[0x4] = true,
                Button::Keyboard(Key::W) => keyboard[0x5] = true,
                Button::Keyboard(Key::E) => keyboard[0x6] = true,
                Button::Keyboard(Key::R) => keyboard[0x7] = true,
                Button::Keyboard(Key::A) => keyboard[0x8] = true,
                Button::Keyboard(Key::S) => keyboard[0x9] = true,
                Button::Keyboard(Key::D) => keyboard[0xA] = true,
                Button::Keyboard(Key::F) => keyboard[0xB] = true,
                Button::Keyboard(Key::Z) => keyboard[0xC] = true,
                Button::Keyboard(Key::X) => keyboard[0xD] = true,
                Button::Keyboard(Key::C) => keyboard[0xE] = true,
                Button::Keyboard(Key::V) => keyboard[0xF] = true,

                Button::Keyboard(Key::P) => machine.int(),
                //Button::Keyboard(Key::N) => machine.tick(&keyboard),
                _ => {}
            }
        }

        if let Some(btn) = e.release_args() {
            match btn {
                Button::Keyboard(Key::D1) => keyboard[0x0] = false,
                Button::Keyboard(Key::D2) => keyboard[0x1] = false,
                Button::Keyboard(Key::D3) => keyboard[0x2] = false,
                Button::Keyboard(Key::D4) => keyboard[0x3] = false,
                Button::Keyboard(Key::Q) => keyboard[0x4] = false,
                Button::Keyboard(Key::W) => keyboard[0x5] = false,
                Button::Keyboard(Key::E) => keyboard[0x6] = false,
                Button::Keyboard(Key::R) => keyboard[0x7] = false,
                Button::Keyboard(Key::A) => keyboard[0x8] = false,
                Button::Keyboard(Key::S) => keyboard[0x9] = false,
                Button::Keyboard(Key::D) => keyboard[0xA] = false,
                Button::Keyboard(Key::F) => keyboard[0xB] = false,
                Button::Keyboard(Key::Z) => keyboard[0xC] = false,
                Button::Keyboard(Key::X) => keyboard[0xD] = false,
                Button::Keyboard(Key::C) => keyboard[0xE] = false,
                Button::Keyboard(Key::V) => keyboard[0xF] = false,

                _ => {}
            }
        }
        //println!("Input: {}us", _now.elapsed().as_micros());
        machine.tick(&keyboard);

        if !machine.rendered {
            //continue;
        }

        let _now = Instant::now();

        if e.render_args().is_some() {
            for y in 0..32 {
                for x in 0..64 {
                    let value = machine.video_memory[y * 64 + x];
                    let color = if value { [255; 4] } else { [0; 4] };
                    for j in 0..CELL_PIXEL_SIDE {
                        let sy = y as u32 * CELL_PIXEL_SIDE + j;
                        for i in 0..CELL_PIXEL_SIDE {
                            let sx = x as u32 * CELL_PIXEL_SIDE + i;
                            canvas.put_pixel(sx, sy, im::Rgba(color));
                        }
                    }
                }
            }
        }


        texture.update(&mut ctx, &canvas).unwrap();
        window.draw_2d(&e, |c, g, d| {
            ctx.encoder.flush(d);
            clear([0., 0., 0., 1.], g);
            image(&texture, c.transform, g);
        });
        //println!("Render: {}ms", _now.elapsed().as_millis());
    }
}

use std::fs::File;
use std::io::Read;

fn read_file(path: &str) -> [u8; 3584] {
    let mut f = File::open(path).unwrap();
    let mut buffer = [0u8; 3584];
    let _len = f.read(&mut buffer).unwrap();
    buffer
}
