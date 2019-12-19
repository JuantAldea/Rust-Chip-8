use sdl2::render::{Canvas, RenderTarget};

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
extern crate spin_sleep;
const CELL_PIXEL_SIDE: u32 = 20;
const X_SIZE: u32 = CELL_PIXEL_SIDE * 64;
const Y_SIZE: u32 = CELL_PIXEL_SIDE * 32;

/*
let color_on: Color = Color::RGB(255, 255, 255);
let color_off: Color = Color::RGB(0, 0, 0);
*/

pub mod chip8;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("ROM missing!");
        return;
    }

    let sdl_context = sdl2::init().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let buzzer = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();

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

    //600hz
    let default_clock_cycle = Duration::new(0, 10_000_000 / 6);
    let mut current_clock_cycle = default_clock_cycle;

    'event_loop: loop {
        let looping_time = Instant::now();
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
                    keycode: Some(Keycode::KpPlus),
                    ..
                } => current_clock_cycle /= 2,

                Event::KeyDown {
                    keycode: Some(Keycode::KpMinus),
                    ..
                } => current_clock_cycle *= 2,

                Event::KeyDown {
                    keycode: Some(Keycode::Kp0),
                    ..
                } => current_clock_cycle = default_clock_cycle,

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

        process_sound(&machine, &buzzer);
        machine.tick_clock(&keyboard);

        if machine.video_memory_tainted {
            draw_canvas(&machine, &mut canvas);
        }

        canvas.present();
        let elapsed_time = looping_time.elapsed();

        //debug!("Cycle: {}us, ", elapsed_time.as_micros());

        if let Some(sleep_required) = current_clock_cycle.checked_sub(elapsed_time) {
            let slept_time = Instant::now();
            spin_sleep::sleep(sleep_required);
            debug!(
                "Loop time: {}us, Sleep Required: {}us, Slept for: {}us, Error: {}us",
                elapsed_time.as_micros(),
                sleep_required.as_micros(),
                slept_time.elapsed().as_micros(),
                sleep_required.as_micros() as i64 - slept_time.elapsed().as_micros() as i64,
            );
        }
    }
}

pub fn process_sound<T: sdl2::audio::AudioCallback>(
    machine: &Chip8,
    device: &sdl2::audio::AudioDevice<T>,
) {
    if machine.st == 0 {
        device.pause();
    } else {
        device.resume();
    }
}

fn draw_canvas<T: RenderTarget>(machine: &Chip8, canvas: &mut Canvas<T>) {
    let color_on: Color = Color::RGB(255, 255, 255);
    let color_off: Color = Color::RGB(0, 0, 0);

    for y in 0..32 {
        for x in 0..64 {
            let pixel_value = machine.get_pixel(x, y);
            let color = if pixel_value { color_on } else { color_off };
            canvas.set_draw_color(color);
            let (px, py) = (x as u32 * CELL_PIXEL_SIDE, y as u32 * CELL_PIXEL_SIDE);
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

use sdl2::audio::{AudioCallback, AudioSpecDesired};
// https://docs.rs/sdl2/0.32.2/sdl2/audio/index.html#example
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
