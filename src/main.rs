mod emulator;
mod soc;

use minifb::{Key, Window, WindowOptions};
use std::{fs::File, io::Read, env};

use crate::emulator::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};

// Window parameters
const SCALE_FACTOR: usize = 3;
const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];

fn main() {
    // get arguments from the command line   
    let mut boot_rom_path = String::new();
    let mut rom_path = String::new();
    let mut debug_opt = false;

    for (index, argument) in env::args().enumerate() {
        match index {
            1 => {
                boot_rom_path = argument.clone();
                println!("boot_rom: {}", boot_rom_path);
            }
            2 => {
                rom_path = argument.clone();
                println!("game_rom: {}", rom_path);
            }
            3 => if argument.eq("--debug") {
                    debug_opt = true;
            }
            _ => {} // nothing to do
        }
    }

    if debug_opt {
        println!("emulator mode: debug");
    } else {
        println!("emulator mode: normal");
    }

    // let mut file = File::open("../gb-test-roms/cpu_instrs/cpu_instrs.gb").unwrap();
    let mut file = File::open(boot_rom_path).unwrap();
    let mut bin_data = [0xFF as u8; 256];
    if let Err(message) = file.read_exact(&mut bin_data) {
        panic!("Cannot read file with error message: {}", message);
    }

    let mut rom_file = File::open(rom_path).unwrap();
    let mut rom_data = [0xFF as u8; 32768];
    if let Err(message) = rom_file.read_exact(&mut rom_data) {
        panic!("Cannot read file with error message: {}", message);
    }
    println!("rom file len: {:#06x}", rom_file.metadata().unwrap().len());

    // create the emulated system
    let mut emulator = Emulator::new(&bin_data, &rom_data);

    // run the emulator
    manage_window(&mut emulator);
}

fn manage_window(emulator: &mut Emulator) {
    let mut buffer = [0; SCREEN_HEIGHT * SCREEN_WIDTH];

    let mut window = Window::new(
        "Rustboy",
        WINDOW_DIMENSIONS[0],
        WINDOW_DIMENSIONS[1],
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // run emulator until a new frame is ready
        emulator.run();

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