mod peripheral;
mod cpu;
pub mod gpu;
mod nvic;
mod timer;
mod bootrom;

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
}