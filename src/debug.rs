use crate::emulator::{Emulator, EmulatorState, ONE_FRAME_IN_NS};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum DebuggerCommand {
    HALT,
    RUN,
    STEP,
}

pub enum DebuggerState {
    HALT,
    RUN,
    STEP,
}

pub fn run_debug_mode(emulator: &mut Emulator, dbg_cmd: &mut Vec<DebuggerCommand>) {
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