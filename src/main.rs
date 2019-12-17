use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};
extern crate env_logger;
extern crate log;
use crate::chip8::Chip8;
use log::debug;
use std::env;

const CELL_PIXEL_SIDE: u32 = 20;
const X_SIZE: u32 = CELL_PIXEL_SIDE * 64;
const Y_SIZE: u32 = CELL_PIXEL_SIDE * 32;

pub mod chip8;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("ROM missing!");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8", X_SIZE, Y_SIZE)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut machine = Chip8::new();
    machine.init();

    let rom = read_file(&args[1]);

    machine.load_rom(0x200, &rom);
    machine.set_pc(0x200);

    let mut keyboard = [false; 16];

    let mut event_pump = sdl_context.event_pump().unwrap();

    'event_loop: loop {
        let now = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'event_loop,

                Event::KeyDown {
                    keycode: Some(Keycode::O),
                    ..
                } => {
                    machine = Chip8::default();
                    machine.init();
                    machine.load_rom(0x200, &rom);
                    machine.set_pc(0x200);
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => handle_key(&mut keyboard, &mut machine, keycode, true),

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => handle_key(&mut keyboard, &mut machine, keycode, false),

                _ => {}
            }
        }

        machine.tick_clock(&keyboard);

        if machine.video_memory_tainted {
            for y in 0..32 {
                for x in 0..64 {
                    let pixel_on = machine.video_memory[y * 64 + x];
                    let color = if pixel_on {
                        Color::RGB(255, 255, 255)
                    } else {
                        Color::RGB(0, 0, 0)
                    };
                    canvas.set_draw_color(color);
                    let px = x as u32 * CELL_PIXEL_SIDE;
                    let py = y as u32 * CELL_PIXEL_SIDE;
                    canvas
                        .fill_rect(Rect::new(
                            px as i32,
                            py as i32,
                            CELL_PIXEL_SIDE,
                            CELL_PIXEL_SIDE,
                        ))
                        .unwrap();
                }
            }
        }

        canvas.present();
        let elapsed_time = now.elapsed();

        debug!("Cycle: {}us, ", elapsed_time.as_micros());
        // 1 second / 600Hz - cycle_time
        if let Some(duration) = Duration::new(0, 10_000_000 / 6).checked_sub(elapsed_time) {
            debug!("Sleep {}us", duration.as_micros());
            ::std::thread::sleep(duration);
        }
    }
}

fn read_file(path: &str) -> [u8; 3584] {
    let mut f = File::open(path).unwrap();
    let mut buffer = [0u8; 3584];
    let _len = f.read(&mut buffer).unwrap();
    buffer
}

pub fn handle_key(
    pad_state: &mut [bool; 16],
    machine: &mut Chip8,
    keycode: Keycode,
    pressed: bool,
) {
    /*
        HEX PAD | QWERTY
        1 2 3 C | 1 2 3 4
        4 5 6 D | Q W E R
        7 8 9 E | A S D F
        A 0 B F | Z X C V
    */

    match keycode {
        // Keypad keys
        Keycode::Num1 => pad_state[0x1] = pressed,
        Keycode::Num2 => pad_state[0x2] = pressed,
        Keycode::Num3 => pad_state[0x3] = pressed,
        Keycode::Num4 => pad_state[0xC] = pressed,

        Keycode::Q => pad_state[0x4] = pressed,
        Keycode::W => pad_state[0x5] = pressed,
        Keycode::E => pad_state[0x6] = pressed,
        Keycode::R => pad_state[0xD] = pressed,

        Keycode::A => pad_state[0x7] = pressed,
        Keycode::S => pad_state[0x8] = pressed,
        Keycode::D => pad_state[0x9] = pressed,
        Keycode::F => pad_state[0xE] = pressed,

        Keycode::Z => pad_state[0xA] = pressed,
        Keycode::X => pad_state[0x0] = pressed,
        Keycode::C => pad_state[0xB] = pressed,
        Keycode::V => pad_state[0xF] = pressed,

        // other keys
        Keycode::P => {
            if pressed {
                machine.int()
            }
        }
        Keycode::N => {
            if pressed {
                machine.next(pad_state)
            }
        }
        _ => {}
    }
}
