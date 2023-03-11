pub mod peripheral;
mod cpu;

use cpu::Cpu;
use peripheral::Peripheral;
use crate::cartridge::Cartridge;

const CLOCK_TICK_PER_MACHINE_CYCLE: u8 = 4;

pub struct Soc {
    pub cpu: Cpu,
    pub peripheral: Peripheral,
}

impl Soc {
    pub fn new(cartridge: Cartridge) -> Soc {
        Soc {
            cpu: Cpu::new(),
            peripheral: Peripheral::new(cartridge),
        }
    }

    pub fn run(&mut self) -> u8 {
        let cycles = self.cpu.run(&mut self.peripheral);

        self.peripheral.run(cycles * CLOCK_TICK_PER_MACHINE_CYCLE);

        cycles
    }

    pub fn load(&mut self, boot_rom: &[u8]) {
        self.peripheral.load_bootrom(boot_rom);
    }

    pub fn get_frame_buffer(&self, pixel_index: usize) -> u8 {
        self.peripheral.gpu.frame_buffer[pixel_index]
    }
}