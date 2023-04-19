mod emulator;
mod soc;
mod debug;
mod cartridge;

use winit::{
    event::{Event, WindowEvent, ElementState, DeviceEvent, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::PhysicalSize,
};
use softbuffer::GraphicsContext;

use std::{fs::File, io::Read, env};
use std::sync::{Arc, Mutex};

use crate::emulator::{Emulator, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::debug::{DebugCtx, debug_cli, debug_vram};

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

    // launch the debugger cli
    let dbg_ctx = Arc::new(Mutex::new(DebugCtx::new()));
    if debug_mode {
        debug_cli(&dbg_ctx);
        debug_vram(&dbg_ctx);
    }

    // create the emulated system
    let mut emulator = Emulator::new(&bin_data, &rom_data, debug_mode);

    // run the emulator
    let mut buffer = [0; SCREEN_HEIGHT * SCREEN_WIDTH];

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
                            .with_inner_size(PhysicalSize::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))
                            .build(&event_loop)
                            .unwrap();
    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    event_loop.run(move |event, _, control_flow| {
        // run the emulator
        emulator.run(&mut *dbg_ctx.lock().unwrap());

        // if a new frame is read, update the framebuffer
        if emulator.frame_ready() {
            // copy the emulator frame buffer to the gpu frame buffer
            for i in 0..SCREEN_HEIGHT * SCREEN_WIDTH {
                buffer[i] = 255 << 24
                            | (emulator.get_frame_buffer(i) as u32) << 16
                            | (emulator.get_frame_buffer(i) as u32) << 8
                            | (emulator.get_frame_buffer(i) as u32) << 0;
            }
        }

        // handle window events
        *control_flow = ControlFlow::Poll;
        match event {
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                match event {
                    DeviceEvent::Key(KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    }) => match key {
                        VirtualKeyCode::Up => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::UP, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::UP, false);
                            }
                        }
                        VirtualKeyCode::Down => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::DOWN, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::DOWN, false);
                            }
                        }
                        VirtualKeyCode::Left => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::LEFT, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::LEFT, false);
                            }
                        }
                        VirtualKeyCode::Right => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::RIGHT, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::RIGHT, false);
                            }
                        }
                        VirtualKeyCode::Return => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::START, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::START, false);
                            }
                        }
                        VirtualKeyCode::Space => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::SELECT, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::SELECT, false);
                            }
                        }
                        VirtualKeyCode::A => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::A, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::A, false);
                            }
                        }
                        VirtualKeyCode::S => {
                            if *state == ElementState::Pressed {
                                emulator.set_key(soc::GameBoyKey::B, true);
                            } else {
                                emulator.set_key(soc::GameBoyKey::B, false);
                            }
                        }
                        _ => {},
                    }
                    _ => {},
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                graphics_context.set_buffer(&buffer, width as u16, height as u16);
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
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