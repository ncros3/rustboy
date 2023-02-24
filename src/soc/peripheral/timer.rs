use crate::soc::peripheral::nvic::{Nvic, InterruptSources};

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
    // timer values
    pub divider: u8,
    pub value: u8,
    pub modulo: u8,
    // control register
    pub main_timer_frequency: Frequency,
    pub divider_timer_frequency: Frequency,
    pub enabled: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            main_timer_cycles: 0,
            divider_timer_cycles: 0,
            divider: 0,
            value: 0,
            modulo: 0,
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

            // divide the main cpu clock
            let main_cycles_per_tick = self.main_timer_frequency.cycles_per_tick();
            if self.main_timer_cycles > main_cycles_per_tick {
                self.main_timer_cycles = self.main_timer_cycles % main_cycles_per_tick;

                // check if the main timer reached its maximum value
                let (new_value, overflow) = self.value.overflowing_add(1);
                self.value = new_value;

                if overflow {
                    nvic.set_interrupt(InterruptSources::TIMER);
                    self.value = self.modulo;
                }
            } 

            // check if the divider timer reached its maximum value
            let divider_cycles_per_tick = self.divider_timer_frequency.cycles_per_tick();
            if self.divider_timer_cycles > divider_cycles_per_tick {
                self.divider_timer_cycles = self.divider_timer_cycles % divider_cycles_per_tick;

                // check if the main timer reached its maximum value
                let (new_divider, overflow) = self.divider.overflowing_add(1);
                self.divider = new_divider;
            } 
        }
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

        for cycles in 0..=1024 {
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

        for cycles in 0..=1024 {
            timer.run(1, &mut nvic);
        }

        assert_eq!(timer.value, 0x00);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::TIMER);

        timer.modulo = 0xF5;
        timer.value = 0xFF;

        for cycles in 0..=1024 {
            timer.run(1, &mut nvic);
        }

        assert_eq!(timer.value, 0xF5);
    }
}