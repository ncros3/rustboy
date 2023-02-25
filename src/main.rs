mod emulator;
mod soc;

use minifb::{Key, Window, WindowOptions};
use std::{fs::File, io::Read};

use crate::emulator::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};

// Window parameters
const SCALE_FACTOR: usize = 3;
const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];

fn main() {
    // let mut file = File::open("../gb-test-roms/cpu_instrs/cpu_instrs.gb").unwrap();
    let mut file = File::open("../dmg_boot.bin").unwrap();
    let mut bin_data = [0xFF as u8; 256];
    file.read_exact(&mut bin_data);

    let mut rom_file = File::open("../gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb").unwrap();
    let mut rom_data = [0xFF as u8; 32768];
    rom_file.read_exact(&mut rom_data);
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