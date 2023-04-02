pub mod peripheral;
mod cpu;

use cpu::Cpu;
use peripheral::Peripheral;
use crate::cartridge::Cartridge;
pub use peripheral::keypad::GameBoyKey;

const CLOCK_TICK_PER_MACHINE_CYCLE: u8 = 4;

pub struct Soc {
    pub cpu: Cpu,
    pub peripheral: Peripheral,
}

impl Soc {
    pub fn new(boot_rom: &[u8], cartridge: Cartridge) -> Soc {
        let mut peripheral = Peripheral::new(cartridge);
        peripheral.load_bootrom(boot_rom);

        Soc {
            cpu: Cpu::new(),
            peripheral: peripheral,
        }
    }

    pub fn run(&mut self) -> u8 {
        let cycles = self.cpu.run(&mut self.peripheral) * CLOCK_TICK_PER_MACHINE_CYCLE;

        self.peripheral.run(cycles);

        cycles
    }

    pub fn get_frame_buffer(&self, pixel_index: usize) -> u8 {
        self.peripheral.gpu.frame_buffer[pixel_index]
    }

    pub fn get_vram_buffer(&self, pixel_index: usize) -> u8 {
        // compute tile index
        let tile_index = pixel_index / 64;
        // compute VRAM address from pixel_index
        let row_offset = (pixel_index % 64) / 8;
        // get row for the needed pixel
        let data_0 = self.peripheral.gpu.vram[tile_index * 16 + row_offset * 2];
        let data_1 = self.peripheral.gpu.vram[tile_index * 16 + row_offset * 2 + 1];
        // get pixel bits
        let bit_0 = data_0 >> (7 - (pixel_index % 8)) & 0x01;
        let bit_1 = data_1 >> (7 - (pixel_index % 8)) & 0x01;

        self.peripheral.gpu.get_bg_pixel_color_from_palette((bit_1 << 1) | bit_0)
    }

    pub fn set_key(&mut self, key: GameBoyKey, value: bool) {
        self.peripheral.keypad.set(key, value);
    }
}