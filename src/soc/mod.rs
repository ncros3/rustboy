mod peripheral;
mod cpu;

use cpu::Cpu;
use peripheral::Peripheral;

pub struct Soc {
    pub cpu: Cpu,
    pub peripheral: Peripheral,
}

impl Soc {
    pub fn new() -> Soc {
        Soc {
            cpu: Cpu::new(),
            peripheral: Peripheral::new(),
        }
    }

    pub fn run(&mut self) -> u8 {
        let cycles = self.cpu.run(&mut self.peripheral);

        self.peripheral.run(cycles);

        cycles
    }

    pub fn load(&mut self, boot_rom: &[u8], rom: &[u8]) {
        self.peripheral.load_bootrom(boot_rom);
        self.peripheral.load_rom(rom);
    }

    pub fn get_frame_buffer(&self, pixel_index: usize) -> u8 {
        self.peripheral.gpu.frame_buffer[pixel_index]
    }
}