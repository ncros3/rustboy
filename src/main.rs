mod peripheral;
mod cpu;
mod gpu;
mod nvic;
mod timer;
mod bootrom;

use minifb::{Key, Window, WindowOptions};
use std::{fs::File, io::Read};
use std::time::Instant;

use cpu::Cpu;
use gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

const BOOTROM: [u8; 256] = [
    // init_stack:
    0x31, 0xFE, 0xFF, // LD     SP 0xfffe
    0xAF,             // XOR    A A
    0x21, 0xFF, 0x9F, // LD     HL #VRAM_END

    // clear_vram:
    0x32,             // LDD    [HL] A
    0xCB, 0x7C,       // BIT    H 7
    0x20, 0xFD,       // JR NZ  clear_vram

    // ***************   INIT AUDIO   ********************
    // init_sound:
    0x21, 0x26, 0xFF, // LD     HL 0xff26
    0x0E, 0x11,       // LD     C 0x11
    0x3E, 0x80,       // LD     A 0x80
    0x32,             // LDD    [HL] A
    0xE2,             // LD     [0xff00 + C] A
    0x0C,             // INC    C
    0x3E, 0xF3,       // LD     A 0xf3
    0xE2,             // LD     [0xff00 + C] A
    0x32,             // LDD    [HL] A
    0x3E, 0x77,       // LD     A 0x77
    0x77,             // LD     [HL] A

    // ************   SETUP BG PALETTE   *****************
    // init_palette:
    0x3E, 0xFC,       // LD     A 0xfc
    0xE0, 0x47,       // LD     [0xff00 + #BGP] A

    // *************   LOAD LOGO DATA   ******************
    // load logo:
    0x11, 0xA8, 0x00, // LD     DE 0x00A8 // nintendo logo
    0x21, 0x10, 0x80, // LD     HL 0x8010
    // convert logo data:
    0x1A,             // LD     A [DE]
    0xCD, 0x95, 0x00, // CALL   graphic_routine_1
    0xCD, 0x96, 0x00, // CALL   graphic_routine_2
    0x13,             // INC    DE
    0x7B,             // LD     A E
    0xFE, 0x34,       // CP     A 0x34
    0x20, 0xF5,       // JR NZ  convert_logo_data

    // **********   LOAD TRADEMARK DATA   ***************
    // load trademark data:
    0x11, 0xD8, 0x00, // LD     DE #tile_data
    0x06, 0x08,       // LD     B 8
    // convert trademark data:
    0x1A,             // LD     A [DE]
    0x13,             // INC    DE
    0x22,             // LDI    [HL] A
    0x23,             // INC    HL
    0x05,             // DEC    B
    0x20, 0xFB,       // JR NZ  copy_tile_map

    // *************   COPY TILEMAP   ******************
    // copy tile map
    0x3E, 0x19,       // LD     A 0x19
    0xEA, 0x10, 0x99, // LD     [0x9910] A
    0x21, 0x2F, 0x99, // LD     HL 0x992f
    // copy 2 lines of 12 tiles
    0x0E, 0x0C,       // LD     C 0x0c
    // copy 12 tiles line
    0x3D,             // DEC    A
    0x28, 0x0A,       // JR Z   init_scroll
    0x32,             // LDD    [HL] A
    0x0D,             // DEC    C
    0x20, 0xFB,       // JR NZ  init_tiles_inner
    0x2E, 0x0F,       // LD     L 0x0f
    0x18, 0xF5,       // JR     init_tiles_loop

    // *********   LOGO SCROLL ROUTINE   **************
    // init_scroll:
    0x67,             // LD     H A
    0x3E, 0x64,       // LD     A 0x64
    0x57,             // LD     D A
    0xE0, 0x42,       // LD     [0xff00 + #SCY] A
    0x3E, 0x91,       // LD     A 0x91
    0xE0, 0x40,       // LD     [0xff00 + #LCDC] A
    0x04,             // INC    B
    // scroll_loop:
    0x1E, 0x02,       // LD     E 0x02
    // wait_next_vblank:
    0x0E, 0x0C,       // LD     C 0x0c
    // wait_vblank:
    0xF0, 0x44,       // LD     A [0xff00 + #LY]
    0xFE, 0x91,       // CP     A 0x90
    0x20, 0xFC,       // JR NZ  wait_vblank

    0x0D,             // DEC    C
    0x20, 0xF9,       // JR NZ  wait_vblank
    0x1D,             // DEC    E
    0x20, 0xF4,       // JR NZ  wait_next_vblank

    0x0E, 0x13,       // LD     C 0x13
    0x24,             // INC    H
    0x7C,             // LD     A H
    0x1E, 0x83,       // LD     E 0x83
    0xFE, 0x62,       // CP     A 0x62
    0x28, 0x08,       // JR Z   play_sound
    0x1E, 0xC1,       // LD     E 0xc1
    0xFE, 0x64,       // CP     A 0x64
    0x20, 0x08,       // JR NZ  skip_sound
    // play_sound:
    0x7B,             // LD     A E
    0xE2,             // LD     [0xff00 + C] A
    0x0C,             // INC    C
    0x3E, 0x87,       // LD     A 0x87
    0xE2,             // LD     [0xff00 + C] A
    // skip_sound:
    0xF0, 0x42,       // LD     A [0xff00 + #SCY]
    0x90,             // SUB    A B
    0xE0, 0x42,       // LD     [0xff00 + #SCY] A
    0x15,             // DEC    D
    0x20, 0xD4,       // JR NZ  scroll_loop
    0x05,             // DEC    B
    0x20, 0x51,       // JN NZ  validate_cart
    0x16, 0x20,       // LD     D 0x20
    0x18, 0xCD,       // JR     scroll_loop

    // **********   GRAPHIC ROUTINE   ***************
    // graphic_routine_1:
    0x4F,             // LD     C A
    // graphic_routine_2:
    0x06, 0x04,       // LD     B 0x04
    // graphic_sub_routine:
    0xC5,             // PUSH   BC
    0xCB, 0x11,       // RL     C
    0x17,             // RL     A
    0xC1,             // POP    BC
    0xCB, 0x11,       // RL     C
    0x17,             // RL     A
    0x05,             // DEC    B
    0x20, 0xF7,       // JR NZ  graphic_sub_routine

    0x22,             // LDI    [HL] A
    0x23,             // INC    HL
    0x22,             // LDI    [HL] A
    0x23,             // INC    HL
    0xC9,             // RET

    // **********   LOGO DATA BYTES   ***************
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
    0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,

    // ********   TRADEMARK DATA BYTES   *************
    0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,

    // *********   VALIDATE CARTRIDGE   **************
    // validate_cart:
    0x21, 0x04, 0x01, // LD     HL, 0x0104
    0x11, 0xA8, 0x00, // LD     DE, #expected_csum
    // checksum_check:
    0x1A,             // LD     A, [DE]
    0x13,             // INC    DE
    0xBE,             // CP     A, [HL]
    // This is an infinite loop when the checksum fails. Replacing
    // it with 0x00 0x00 (NOP NOP) will allow invalid ROMs to run.
    0x20, 0xFE,       // JR NZ  .
    0x23,             // INC    HL
    0x7D,             // LD     A L
    0xFE, 0x34,       // CP     A 0x34
    0x20, 0xF7,       // JR NZ  checksum_check
    0x06, 0x19,       // LD     B 0x19
    0x78,             // LD     A B
    // header_sum
    0x86,             // ADD    A [HL]
    0x23,             // INC    HL
    0x05,             // DEC    B
    0x20, 0xFD,       // JR NZ  header_sum
    0x86,             // ADD    A [HL]
    // same as above, infinite loop if the sum is bad, replace with
    // NOPs to run anyway.
    0x20, 0xFE,       // JR NZ  .
    0x3E, 0x01,       // LD A   0x1
    // There shouldn't be anything at that address, I assume that's
    // how you tell the hardware to unmap the bootrom
    0xE0, 0x50,       // LD [0xff00 + 0x50] A
];

fn main() {
    // create the emulated system
    let mut cpu = Cpu::new();

    // let mut file = File::open("../gb-test-roms/cpu_instrs/cpu_instrs.gb").unwrap();
    let mut file = File::open("../dmg_boot.bin").unwrap();
    let mut bin_data = [0xFF as u8; 256];
    file.read_exact(&mut bin_data);

    let mut rom_file = File::open("../gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb").unwrap();
    let mut rom_data = [0xFF as u8; 32768];
    rom_file.read_exact(&mut rom_data);
    println!("rom file len: {:#06x}", rom_file.metadata().unwrap().len());

    cpu.peripheral.load_bootrom(&bin_data);
    cpu.peripheral.load_rom(&rom_data);


    // run the emulator
    emulator_run(&mut cpu);
}

fn emulator_run(cpu: &mut Cpu) {
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
                cycles_elapsed_in_frame += cpu.run() as usize;

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
                                | (cpu.peripheral.gpu.frame_buffer[i] as u32) << 16
                                | (cpu.peripheral.gpu.frame_buffer[i] as u32) << 8
                                | (cpu.peripheral.gpu.frame_buffer[i] as u32) << 0;
                }
                // display the frame rendered by the gpu
                window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();

                emulator_state = EmulatorState::GetTime;
            }
        }
    }
}