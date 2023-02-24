mod soc;

use minifb::{Key, Window, WindowOptions};
use std::{fs::File, io::Read};
use std::time::Instant;

use soc::Soc;
use soc::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

// Window parameters
const SCALE_FACTOR: usize = 3;
const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];

// system parameters
const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4194304;
const ONE_FRAME_IN_CYCLES: usize = 70224;
const ONE_FRAME_IN_NS: usize = ONE_FRAME_IN_CYCLES * ONE_SECOND_IN_MICROS / ONE_SECOND_IN_CYCLES;

enum EmulatorState {
    GetTime,
    RunMachine,
    WaitNextFrame,
    DisplayFrame
}

fn main() {
    // create the emulated system
    let mut soc = Soc::new();

    // let mut file = File::open("../gb-test-roms/cpu_instrs/cpu_instrs.gb").unwrap();
    let mut file = File::open("../dmg_boot.bin").unwrap();
    let mut bin_data = [0xFF as u8; 256];
    file.read_exact(&mut bin_data);

    let mut rom_file = File::open("../gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb").unwrap();
    let mut rom_data = [0xFF as u8; 32768];
    rom_file.read_exact(&mut rom_data);
    println!("rom file len: {:#06x}", rom_file.metadata().unwrap().len());

    soc.peripheral.load_bootrom(&bin_data);
    soc.peripheral.load_rom(&rom_data);


    // run the emulator
    emulator_run(&mut soc);
}

fn emulator_run(soc: &mut Soc) {
    let mut buffer = [0; SCREEN_HEIGHT * SCREEN_WIDTH];
    let mut cycles_elapsed_in_frame = 0usize;
    let mut emulator_frame_tick = Instant::now();
    let mut emulator_state = EmulatorState::GetTime;

    let mut window = Window::new(
        "Rustboy",
        WINDOW_DIMENSIONS[0],
        WINDOW_DIMENSIONS[1],
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        match emulator_state {
            EmulatorState::GetTime => {
                emulator_frame_tick = Instant::now();

                emulator_state = EmulatorState::RunMachine;
            }
            EmulatorState::RunMachine => {
                cycles_elapsed_in_frame += soc.run() as usize;

                if cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
                    cycles_elapsed_in_frame = 0;
                    emulator_state = EmulatorState::WaitNextFrame;
                }
            }
            EmulatorState::WaitNextFrame => {
                // check if 16,742706 ms have passed during this frame
                if emulator_frame_tick.elapsed().as_nanos() >= ONE_FRAME_IN_NS as u128{
                    emulator_state = EmulatorState::DisplayFrame;
                }
            }
            EmulatorState::DisplayFrame => {
                // copy the current frame from gpu frame buffer
                for i in 0..SCREEN_HEIGHT * SCREEN_WIDTH {
                    buffer[i] =  255 << 24
                                | (soc.peripheral.gpu.frame_buffer[i] as u32) << 16
                                | (soc.peripheral.gpu.frame_buffer[i] as u32) << 8
                                | (soc.peripheral.gpu.frame_buffer[i] as u32) << 0;
                }
                // display the frame rendered by the gpu
                window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();

                emulator_state = EmulatorState::GetTime;
            }
        }
    }
}