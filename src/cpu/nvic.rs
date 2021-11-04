pub enum InterruptSources {
    VBLANK = 0,
    LCD_STAT = 1,
    TIMER = 2,
    SERIAL = 3,
    JOYPAD = 4,
}

pub struct Nvic {
    pub interrupt_master_enable: bool,
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,
}

impl Nvic {
    pub fn new() -> Nvic {
        Nvic {
            interrupt_master_enable: false,
            interrupt_enable: 0,
            interrupt_flag: 0,
        }
    }

    pub fn master_enable(&mut self, enable: bool) {
        self.interrupt_master_enable = enable;
    }

    pub fn enable_interrupt(&mut self, source: InterruptSources, enable: bool) {
        self.interrupt_enable = (enable as u8) << (source as u8);
    }
}

#[cfg(test)]
mod nvic_tests {
    use super::*;
    use crate::cpu::nvic::InterruptSources::SERIAL;

    #[test]
    fn test_enable_interrupt() {
        let mut nvic = Nvic::new();

        nvic.enable_interrupt(SERIAL, true);
        assert_eq!(nvic.interrupt_enable, 0x08);
    }
}
