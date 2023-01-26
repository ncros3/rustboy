pub enum InterruptSources {
    VBLANK,
    LCD_STAT,
    TIMER,
    SERIAL,
    JOYPAD,
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

    #[test]
    fn test_enable_interrupt() {
        let mut nvic = Nvic::new();

        nvic.enable_interrupt(InterruptSources::VBLANK, true);
        assert_eq!(nvic.interrupt_enable, 0x01);

        nvic.enable_interrupt(InterruptSources::LCD_STAT, true);
        assert_eq!(nvic.interrupt_enable, 0x02);

        nvic.enable_interrupt(InterruptSources::TIMER, true);
        assert_eq!(nvic.interrupt_enable, 0x04);

        nvic.enable_interrupt(InterruptSources::SERIAL, true);
        assert_eq!(nvic.interrupt_enable, 0x08);

        nvic.enable_interrupt(InterruptSources::JOYPAD, true);
        assert_eq!(nvic.interrupt_enable, 0x10);
    }
}
