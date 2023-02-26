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
    DisplayFrame,
}

#[derive(Clone, Copy)]
pub enum DebuggerCommand {
    HALT,
    RUN,
    STEP,
}

enum DebuggerState {
    HALT,
    RUN,
    STEP,
}

pub struct Emulator {
    // gameboy emulated hardware
    soc: Soc,
    // emulator internal parameters
    state: EmulatorState,
    cycles_elapsed_in_frame: usize,
    frame_tick: Instant,
    // debugger parameters
    debugger_state: DebuggerState,
    display_cpu_reg: bool,
    run_routine: fn(&mut Emulator, &mut Vec<DebuggerCommand>),
}

impl Emulator {
    pub fn new(boot_rom: &[u8], rom: &[u8], debug_on: bool) -> Emulator {
        let mut soc = Soc::new();
        soc.load(boot_rom, rom);

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
            debugger_state: DebuggerState::HALT,
            display_cpu_reg: true,
            run_routine: run_routine,
        }
    }

    pub fn run(&mut self, dbg_cmd: &mut Vec<DebuggerCommand>) {
        (self.run_routine)(self, dbg_cmd);
    }

    pub fn step(&mut self) {
        self.cycles_elapsed_in_frame += self.soc.run() as usize;
    
        if self.cycles_elapsed_in_frame >= ONE_FRAME_IN_CYCLES {
            self.cycles_elapsed_in_frame = 0;
            self.state = EmulatorState::WaitNextFrame;
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

fn run_normal_mode(emulator: &mut Emulator, cmd: &mut Vec<DebuggerCommand>) {
    match emulator.state {
        EmulatorState::GetTime => {
            emulator.frame_tick = Instant::now();

            emulator.state = EmulatorState::RunMachine;
        }
        EmulatorState::RunMachine => {
            emulator.step();
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

fn run_debug_mode(emulator: &mut Emulator, dbg_cmd: &mut Vec<DebuggerCommand>) {
    match emulator.state {
        EmulatorState::GetTime => {
            emulator.frame_tick = Instant::now();

            emulator.state = EmulatorState::RunMachine;
        }
        EmulatorState::RunMachine => {
            match emulator.debugger_state {
                DebuggerState::HALT => {
                    // display cpu internal registers
                    display_cpu_reg(emulator);

                    // wait until a new debug command is entered
                    let cmd = dbg_cmd.pop();
                    if let Some(DebuggerCommand::RUN) = cmd {
                        emulator.display_cpu_reg = true;
                        emulator.debugger_state = DebuggerState::RUN;
                    }

                    if let Some(DebuggerCommand::STEP) = cmd {
                        emulator.display_cpu_reg = true;
                        emulator.debugger_state = DebuggerState::STEP;
                    }
                }
                DebuggerState::RUN => {
                    // run the emulator as in normal mode
                    emulator.step();

                    // wait until a new debug command is entered
                    if let Some(DebuggerCommand::HALT) = dbg_cmd.pop() {
                        emulator.display_cpu_reg = true;
                        emulator.debugger_state = DebuggerState::HALT;
                    }
                }
                DebuggerState::STEP => {
                    // run the emulator once then go to halt state
                    emulator.step();

                    emulator.debugger_state = DebuggerState::HALT;
                }
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

fn display_cpu_reg(emulator: &mut Emulator) {
    if emulator.display_cpu_reg {
        emulator.display_cpu_reg = false;
        println!("instruction byte : {:#04x} / pc : {:#06x} / sp : {:#04x}", emulator.soc.peripheral.read(emulator.soc.cpu.pc), emulator.soc.cpu.pc, emulator.soc.cpu.sp);
        println!("BC : {:#06x} / AF : {:#06x} / DE : {:#06x} / HL : {:#06x}", emulator.soc.cpu.registers.read_bc(), emulator.soc.cpu.registers.read_af(), emulator.soc.cpu.registers.read_de(), emulator.soc.cpu.registers.read_hl());
    }
}