use crate::soc::Soc;
use std::time::Instant;

pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_WIDTH: usize = 160;

// emulator clock parameters
const ONE_SECOND_IN_MICROS: usize = 1000000000;
const ONE_SECOND_IN_CYCLES: usize = 4194304; // Main sys clock 4.194304 MHz
const ONE_FRAME_IN_CYCLES: usize = 70224;
const ONE_FRAME_IN_NS: usize = ONE_FRAME_IN_CYCLES * ONE_SECOND_IN_MICROS / ONE_SECOND_IN_CYCLES;

#[derive(PartialEq)]
pub enum EmulatorState {
    GetTime,
    RunMachine,
    WaitNextFrame,
    DisplayFrame
}

pub struct Emulator {
    soc: Soc,
    state: EmulatorState,
    cycles_elapsed_in_frame: usize,
    emulator_frame_tick: Instant,
}

impl Emulator {
    pub fn new(boot_rom: &[u8], rom: &[u8]) -> Emulator {
        let mut soc = Soc::new();
        soc.load(boot_rom, rom);

        Emulator {
            soc: soc,
            state: EmulatorState::GetTime,
            cycles_elapsed_in_frame: 0 as usize,
            emulator_frame_tick: Instant::now(),
        }
    }

    pub fn run(&mut self) {
        match self.state {
            EmulatorState::GetTime => {
                self.emulator_frame_tick = Instant::now();

                self.state = EmulatorState::RunMachine;
            }
            EmulatorState::RunMachine => {
                self.cycles_elapsed_in_frame += self.soc.run() as usize;

                if self.cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
                    self.cycles_elapsed_in_frame = 0;
                    self.state = EmulatorState::WaitNextFrame;
                }
            }
            EmulatorState::WaitNextFrame => {
                // check if 16,742706 ms have passed during this frame
                if self.emulator_frame_tick.elapsed().as_nanos() >= ONE_FRAME_IN_NS as u128{
                    self.state = EmulatorState::DisplayFrame;
                }
            }
            EmulatorState::DisplayFrame => {
                self.state = EmulatorState::GetTime;
            }
        }
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
}