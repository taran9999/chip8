extern crate sdl2;

mod cpu;
mod sound;

use std::env;
use std::time::{Duration, Instant};
use std::{fs::File, io::Read};

use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: usize = 20;
const IPS: u64 = 700; // instructions per second
const COLOR_1: Color = Color::WHITE;
const COLOR_2: Color = Color::BLACK;

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname = &args[1];
    println!("fname: {fname}");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer = sdl_context.timer().unwrap();
    let audio = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(1024),
    };

    let audio_device = audio
        .open_playback(None, &desired_spec, |spec| sound::SquareWave {
            phase: 0.0,
            phase_increment: 440.0 / spec.freq as f32,
            volume: 0.25,
        })
        .unwrap();

    let window = video_subsystem
        .window(
            &format!("chip8 | {fname}"),
            WIDTH as u32 * SCALE as u32,
            HEIGHT as u32 * SCALE as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(COLOR_2);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let gap_time = Duration::from_millis(1000 / IPS);
    let mut last_cycle = Instant::now();
    let mut last_timer_tick = 0;
    let mut paused = false;

    let mut chip8 = cpu::Chip8::new(true);
    chip8.init();

    // let test_bin: [u8; 10] = [
    //     0x00, 0xE0, // Clear screen
    //     0x60, 0x05, // Set V0 to 05
    //     0x71, 0x10, // Add 10 to V1
    //     0xA0, 0x02, // Set i to 002
    //     0x12, 0x00 // Jump to 200
    // ];

    // chip8.load_bin(&test_bin);

    let mut file = File::open(&format!("binaries/{fname}")).expect("Error opening file.");
    let mut file_buffer = Vec::new();
    file.read_to_end(&mut file_buffer)
        .expect("Error reading file.");
    chip8.load_bin(&file_buffer);

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("Quitting app");
                    break 'main;
                }

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    chip8.key_down(key);
                    if paused {
                        paused = false;
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    chip8.key_up(key);
                    if paused {
                        paused = false;
                    }
                }

                _ => {}
            }
        }

        let curr_timer = timer.ticks();
        if curr_timer - last_timer_tick >= 1000 / 60 {
            chip8.decrement_timers();
            if chip8.sound_timer() == 0 {
                audio_device.pause();
            }
            last_timer_tick = curr_timer;
        }

        if !paused && last_cycle.elapsed() >= gap_time {
            last_cycle = Instant::now();
            let op = chip8.fetch();
            match chip8.execute(op) {
                cpu::ExecutionEffect::NoEffect => {}

                cpu::ExecutionEffect::DisplayUpdate => {
                    let display = chip8.display();
                    render(&mut canvas, display).unwrap();
                }

                cpu::ExecutionEffect::JumpToSelf => {
                    println!("Jump to self: pausing execution");
                    paused = true;
                }

                cpu::ExecutionEffect::WaitingForKey => {
                    println!("Waiting for key press: pausing execution");
                    paused = true;
                }

                cpu::ExecutionEffect::Sound => {
                    audio_device.resume();
                }
            }
        }
    }
}

fn render(canvas: &mut Canvas<Window>, display: &[u8; WIDTH * HEIGHT]) -> Result<(), String> {
    canvas.set_draw_color(COLOR_2);
    canvas.clear();
    canvas.set_draw_color(COLOR_1);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let px = display[y * WIDTH + x];
            if px != 0 {
                // Draw the pixel as a scaled rectangle
                let rect = Rect::new(
                    (x * SCALE) as i32,
                    (y * SCALE) as i32,
                    SCALE as u32,
                    SCALE as u32,
                );
                canvas.fill_rect(rect)?;
            }
        }
    }

    canvas.present();
    Ok(())
}

