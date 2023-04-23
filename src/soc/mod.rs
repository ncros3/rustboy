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

    pub fn set_key(&mut self, key: GameBoyKey, value: bool) {
        self.peripheral.keypad.set(key, value);
    }
}