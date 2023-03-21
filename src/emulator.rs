use crate::soc::Soc;
pub use crate::soc::GameBoyKey;
use crate::cartridge::Cartridge;
use std::time::Instant;
use crate::debug::{DebugCtx, run_debug_mode};

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;

// emulator clock parameters
const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4194304; // Main sys clock 4.194304 MHz
pub const ONE_FRAME_IN_CYCLES: usize = 70224;
pub const ONE_FRAME_IN_NS: usize = ONE_FRAME_IN_CYCLES * ONE_SECOND_IN_MICROS / ONE_SECOND_IN_CYCLES;

#[derive(PartialEq)]
pub enum EmulatorState {
    GetTime,
    RunMachine,
    WaitNextFrame,
    DisplayFrame,
}

pub struct Emulator {
    // gameboy emulated hardware
    pub soc: Soc,
    // emulator internal parameters
    pub state: EmulatorState,
    pub cycles_elapsed_in_frame: usize,
    pub frame_tick: Instant,
    run_routine: fn(&mut Emulator, &mut DebugCtx),
}

impl Emulator {
    pub fn new(boot_rom: &[u8], rom: &[u8], debug_on: bool) -> Emulator {
        let cartridge = Cartridge::new(rom);

        let soc = Soc::new(boot_rom, cartridge);

        let run_routine = if debug_on {
            run_debug_mode
        } else {
            run_normal_mode
        };

        Emulator {
            // gameboy emulated hardware
            soc: soc,
            // emulator internal parameters
            state: EmulatorState::GetTime,
            cycles_elapsed_in_frame: 0 as usize,
            frame_tick: Instant::now(),
            // debugger parameters
            run_routine: run_routine,
        }
    }

    pub fn run(&mut self, dbg_cmd: &mut DebugCtx) {
        (self.run_routine)(self, dbg_cmd);
    }

    pub fn frame_ready(&self) -> bool {
        if self.state == EmulatorState::DisplayFrame {
            true
        } else {
            false
        }
    }

    pub fn get_frame_buffer(&self, pixel_index: usize) -> u8 {
        self.soc.get_frame_buffer(pixel_index)
    }

    pub fn set_key(&mut self, key: GameBoyKey, value: bool) {
        self.soc.set_key(key, value);
    }
}

fn run_normal_mode(emulator: &mut Emulator, _dbg_ctx: &mut DebugCtx) {
    match emulator.state {
        EmulatorState::GetTime => {
            emulator.frame_tick = Instant::now();

            emulator.state = EmulatorState::RunMachine;
        }
        EmulatorState::RunMachine => {
            emulator.cycles_elapsed_in_frame += emulator.soc.run() as usize;

            if emulator.cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
                emulator.cycles_elapsed_in_frame = 0;
                emulator.state = EmulatorState::WaitNextFrame;
            }
        }
        EmulatorState::WaitNextFrame => {
            // check if 16,742706 ms have passed during this frame
            if emulator.frame_tick.elapsed().as_nanos() >= ONE_FRAME_IN_NS as u128{
                emulator.state = EmulatorState::DisplayFrame;
            }
        }
        EmulatorState::DisplayFrame => {
            emulator.state = EmulatorState::GetTime;
        }
    }
}

