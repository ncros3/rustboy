use crate::soc::peripheral::nvic::{Nvic, InterruptSources};
use crate::soc::CLOCK_TICK_PER_MACHINE_CYCLE;

pub enum Frequency {
    F4096,
    F16384,
    F262144,
    F65536,
}

impl Frequency {
    // The number of CPU cycles that occur per tick of the clock.
    // This is equal to the number of cpu cycles per second (4194304)
    // divided by the timer frequency.
    fn cycles_per_tick(&self) -> usize {
        match self {
            Frequency::F4096 => 1024,
            Frequency::F16384 => 256,
            Frequency::F262144 => 16,
            Frequency::F65536 => 64,
        }
    }
}

pub struct Timer {
    // internal parameters
    main_timer_cycles: usize,
    divider_timer_cycles: usize,
    tima_overflow: bool,
    // DIV / TIMA / TMA registers
    pub divider: u8,
    pub value: u8,
    pub modulo: u8,
    // TAC registers values
    pub main_timer_frequency: Frequency,
    pub divider_timer_frequency: Frequency,
    pub enabled: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            // internal parameters
            main_timer_cycles: 0,
            divider_timer_cycles: 0,
            tima_overflow: false,
            // DIV / TIMA / TMA registers
            divider: 0,
            value: 0,
            modulo: 0,
            // TAC registers values
            main_timer_frequency: Frequency::F4096,
            divider_timer_frequency: Frequency::F16384,
            enabled: false,
        }
    }

    pub fn run(&mut self, cycles: u8, nvic: &mut Nvic) {
        if self.enabled {
            // update internal timer clock
            self.main_timer_cycles += cycles as usize;
            self.divider_timer_cycles += cycles as usize;

            // delay interrupt by 1 machine cycle / 4 clocks
            if self.tima_overflow && self.has_timer_passed_1_cycle() {
                self.tima_overflow = false;
                nvic.set_interrupt(InterruptSources::TIMER);
                self.value = self.modulo;
            }

            // divide the main cpu clock
            let main_cycles_per_tick = self.main_timer_frequency.cycles_per_tick();
            if self.main_timer_cycles > main_cycles_per_tick {
                let add_timer = (self.main_timer_cycles / main_cycles_per_tick) as u8;
                self.main_timer_cycles = self.main_timer_cycles % main_cycles_per_tick;

                // check if the main timer reached its maximum value
                let (new_value, overflow) = self.value.overflowing_add(add_timer);
                self.value = new_value;

                // register overflow for next cycle if any
                // see https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html 
                if overflow {
                    self.tima_overflow = true;
                    self.value = 0;
                }

                // delay interrupt by 1 machine cycle / 4 clocks
                if self.tima_overflow && self.has_timer_passed_1_cycle() {
                    self.tima_overflow = false;
                    nvic.set_interrupt(InterruptSources::TIMER);
                    self.value = self.modulo;
                }
            } 

            // check if the divider timer reached its maximum value
            let divider_cycles_per_tick = self.divider_timer_frequency.cycles_per_tick();
            if self.divider_timer_cycles > divider_cycles_per_tick {
                let add_divider = (self.divider_timer_cycles / divider_cycles_per_tick) as u8;
                self.divider_timer_cycles = self.divider_timer_cycles % divider_cycles_per_tick;

                // check if the main timer reached its maximum value
                let (new_divider, _overflow) = self.divider.overflowing_add(add_divider);
                self.divider = new_divider;
            } 
        }
    }

    pub fn has_timer_passed_1_cycle(&self) -> bool {
        self.main_timer_cycles / CLOCK_TICK_PER_MACHINE_CYCLE as usize > 0
    }

    pub fn set_divider(&mut self) {
        self.divider = 0;
    }

    pub fn get_divider(&self) -> u8 {
        self.divider
    }

    pub fn set_value(&mut self, data: u8) {
        self.value = data;
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }

    pub fn set_modulo(&mut self, data: u8) {
        self.modulo = data;
    }

    pub fn get_modulo(&self) -> u8 {
        self.modulo
    }

    pub fn settings_from_byte(&mut self, data: u8) {
        // timer enable
        self.enabled = ((data >> 2) & 0x01) != 0;

        // main timer frequency
        self.main_timer_frequency = match data & 0x03 {
            0x00 => Frequency::F4096,
            0x01 => Frequency::F262144,
            0x10 => Frequency::F65536,
            _ => Frequency::F16384,
        };
    }
}

#[cfg(test)]
mod timer_tests {
    use super::*;

    #[test]
    fn test_timer_inc() {
        let mut timer = Timer::new();
        let mut nvic = Nvic::new();

        timer.enabled = true;

        for _ in 0..=1024 {
            timer.run(1, &mut nvic);
        }

        assert_eq!(timer.value, 1);
    }

    #[test]
    fn test_timer_overflow() {
        let mut timer = Timer::new();
        let mut nvic = Nvic::new();

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::TIMER, true);
        timer.enabled = true;
        timer.value = 0xFF;

        for _ in 0..=1024 {
            timer.run(1, &mut nvic);
        }

        assert_eq!(timer.value, 0x00);
        assert_eq!(nvic.get_interrupt(), None);

        // run 1 more machine cycle (4 clocks) to rise the interrupt
        timer.run(CLOCK_TICK_PER_MACHINE_CYCLE, &mut nvic);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::TIMER);

        timer.modulo = 0xF5;
        timer.value = 0xFF;

        for _ in 0..=1024 {
            timer.run(1, &mut nvic);
        }

        assert_eq!(timer.value, 0xF5);
    }
}