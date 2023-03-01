use crate::emulator::{Emulator, EmulatorState, ONE_FRAME_IN_NS};
use std::time::Instant;

use std::io::{stdin, stdout, Write};
use std::thread;
use std::sync::{Arc, Mutex};

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

pub struct DebugCtx {
    cmd: Vec<DebuggerCommand>,
    breakpoint: u16,
    break_enabled: bool,
}

impl DebugCtx {
    pub fn new() -> DebugCtx {
        DebugCtx {
            cmd: Vec::new(),
            breakpoint: 0,
            break_enabled: false,
        }
    }
}

pub fn run_debug_mode(emulator: &mut Emulator, dbg_ctx: &mut DebugCtx) {
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
                    let cmd = dbg_ctx.cmd.pop();
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

                    // check if we have to break
                    if dbg_ctx.break_enabled && (dbg_ctx.breakpoint == emulator.soc.cpu.pc) {
                        // check pc
                        emulator.display_cpu_reg = true;
                        emulator.debugger_state = DebuggerState::HALT;
                    }

                    // wait until a new debug command is entered
                    if let Some(DebuggerCommand::HALT) = dbg_ctx.cmd.pop() {
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

pub fn debug_cli(debug_ctx: &Arc<Mutex<DebugCtx>>) {
    let debug_ctx_ref = Arc::clone(&debug_ctx);
    thread::spawn(move || {
        println!("Rustboy debugger CLI");

        loop {
            // get next instruction from console
            let mut command = String::new();
            command.clear();
            stdout().flush().unwrap();
            stdin().read_line(&mut command).expect("Incorrect string is read.");

            // process command
            if command.trim().contains("break_set") {
                let split: Vec<&str> = command.trim().split(" ").collect();
                let brk_addr = u16::from_str_radix(split[1], 16).unwrap();
                (*debug_ctx_ref.lock().unwrap()).breakpoint = brk_addr;
                (*debug_ctx_ref.lock().unwrap()).break_enabled = true;
            }

            if command.trim().contains("break_reset") {
                (*debug_ctx_ref.lock().unwrap()).break_enabled = false;
            }

            if command.trim().contains("run") {
                (*debug_ctx_ref.lock().unwrap()).cmd.push(DebuggerCommand::RUN);
            }

            if command.trim().contains("halt") {
                (*debug_ctx_ref.lock().unwrap()).cmd.push(DebuggerCommand::HALT);
            }

            if command.trim().contains("step") {
                (*debug_ctx_ref.lock().unwrap()).cmd.push(DebuggerCommand::STEP);
            }

            if command.trim().contains("help") {
                println!("supported commands: break <addr>, run, halt, step");
            }
        }
    });
}