mod emulator;
mod soc;
mod debug;
mod cartridge;

use minifb::{Key, Window, WindowOptions};
use std::{fs::File, io::Read, env};
use std::sync::{Arc, Mutex};

use crate::emulator::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::debug::{DebugCtx, debug_cli};

// Window parameters
const SCALE_FACTOR: usize = 3;
const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];

fn main() {
    // get arguments from the command line   
    let (boot_rom_path, game_rom_path, debug_mode) = parse_args();

    let mut file = File::open(boot_rom_path).unwrap();
    let mut bin_data = [0xFF as u8; 256];
    if let Err(message) = file.read_exact(&mut bin_data) {
        panic!("Cannot read file with error message: {}", message);
    }

    let mut rom_file = File::open(game_rom_path).unwrap();
    let rom_len = rom_file.metadata().unwrap().len();
    let mut rom_data = vec![0xFF as u8; rom_len as usize];
    if let Err(message) = rom_file.read_exact(&mut rom_data) {
        panic!("Cannot read file with error message: {}", message);
    }
    println!("rom file len: {:x}", rom_len);

    // launch the debugger cli
    let dbg_ctx = Arc::new(Mutex::new(DebugCtx::new()));
    if debug_mode {
        debug_cli(&dbg_ctx);
    }

    // create the emulated system
    let mut emulator = Emulator::new(&bin_data, &rom_data, debug_mode);

    // run the emulator
    let mut buffer = [0; SCREEN_HEIGHT * SCREEN_WIDTH];

    let mut window = Window::new(
        "Rustboy",
        WINDOW_DIMENSIONS[0],
        WINDOW_DIMENSIONS[1],
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // get key from the keyboard
        if window.is_key_down(Key::Up) {
            emulator.set_key(soc::GameBoyKey::UP, true);
        } else {
            emulator.set_key(soc::GameBoyKey::UP, false);
        }

        if window.is_key_down(Key::Down) {
            emulator.set_key(soc::GameBoyKey::DOWN, true);
        } else {
            emulator.set_key(soc::GameBoyKey::DOWN, false);
        }

        if window.is_key_down(Key::Left) {
            emulator.set_key(soc::GameBoyKey::LEFT, true);
        } else {
            emulator.set_key(soc::GameBoyKey::LEFT, false);
        }

        if window.is_key_down(Key::Right) {
            emulator.set_key(soc::GameBoyKey::RIGHT, true);
        } else {
            emulator.set_key(soc::GameBoyKey::RIGHT, false);
        }

        if window.is_key_down(Key::A) {
            emulator.set_key(soc::GameBoyKey::A, true);
        } else {
            emulator.set_key(soc::GameBoyKey::A, false);
        }

        if window.is_key_down(Key::B) {
            emulator.set_key(soc::GameBoyKey::B, true);
        } else {
            emulator.set_key(soc::GameBoyKey::B, false);
        }

        if window.is_key_down(Key::Space) {
            emulator.set_key(soc::GameBoyKey::START, true);
        } else {
            emulator.set_key(soc::GameBoyKey::START, false);
        }

        if window.is_key_down(Key::Enter) {
            emulator.set_key(soc::GameBoyKey::SELECT, true);
        } else {
            emulator.set_key(soc::GameBoyKey::SELECT, false);
        }

        // run emulator until a new frame is ready
        emulator.run(&mut *dbg_ctx.lock().unwrap());

        if emulator.frame_ready() {
            // copy the current frame from gpu frame buffer
            for i in 0..SCREEN_HEIGHT * SCREEN_WIDTH {
                buffer[i] =  255 << 24
                            | (emulator.get_frame_buffer(i) as u32) << 16
                            | (emulator.get_frame_buffer(i) as u32) << 8
                            | (emulator.get_frame_buffer(i) as u32) << 0;
            }
            // display the frame rendered by the gpu
            window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
        }
    }
}

fn parse_args() -> (String, String, bool) {
    let mut boot_rom_path = String::new();
    let mut game_rom_path = String::new();
    let mut debug_opt = false;

    for (index, argument) in env::args().enumerate() {
        match index {
            1 => {
                boot_rom_path = argument.clone();
                println!("boot_rom: {}", boot_rom_path);
            }
            2 => {
                game_rom_path = argument.clone();
                println!("game_rom: {}", game_rom_path);
            }
            3 => if argument.eq("--debug") {
                    debug_opt = true;
            }
            _ => {} // nothing to do
        }
    }

    (boot_rom_path, game_rom_path, debug_opt)
}