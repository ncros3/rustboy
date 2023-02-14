#[derive(Copy, Clone)]
pub enum InterruptSources {
    VBLANK,
    LCD_STAT,
    TIMER,
    SERIAL,
    JOYPAD,
}

const FIRST_INTERRUPT_SOURCE: u8 = InterruptSources::VBLANK as u8;
const LAST_INTERRUPT_SOURCE: u8 = InterruptSources::JOYPAD as u8;

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
        if enable {
            self.interrupt_enable |= (1 as u8) << (source as u8);
        } else {
            self.interrupt_enable &= !((1 as u8) << (source as u8));
        }
    }

    pub fn set_interrupt(&mut self, source: InterruptSources) {
        self.interrupt_flag |= (1 as u8) << (source as u8);
    }

    pub fn get_interrupt(&mut self) -> Option<InterruptSources> {
        if self.interrupt_master_enable {
            if self.is_an_interrupt_pending() {
                // we detected an interrupt
                // find the interrupt source and clear the bit flag
                for interrupt_index in FIRST_INTERRUPT_SOURCE..=LAST_INTERRUPT_SOURCE {
                    if (self.interrupt_enable & self.interrupt_flag & (1 << interrupt_index)) != 0 {
                        // clear the interrupt flag
                        self.interrupt_flag &= !(1 << interrupt_index);

                        // high priority interrupt found
                        let interrupt_source = match 1 << interrupt_index {
                            1 => InterruptSources::VBLANK,
                            2 => InterruptSources::LCD_STAT,
                            4 => InterruptSources::TIMER,
                            8 => InterruptSources::SERIAL,
                            16 => InterruptSources::JOYPAD,
                            _ => panic!("Interrupt index exceeded interrupt max number")
                        };

                        return Some(interrupt_source);
                    }
                }
                panic!("Interrupt source not defined")
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_an_interrupt_pending(&self) -> bool {
        if (self.interrupt_enable & self.interrupt_flag) != 0 {
            true
        } else {
            false
        }
    }

    pub fn from_byte(&mut self, data: u8) {
        self.interrupt_enable = data;
    }

    pub fn to_byte(&self) -> u8 {
        0b11100000 | self.interrupt_enable
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
        assert_eq!(nvic.interrupt_enable, 0x03);

        nvic.enable_interrupt(InterruptSources::JOYPAD, true);
        assert_eq!(nvic.interrupt_enable, 0x13);

        nvic.enable_interrupt(InterruptSources::LCD_STAT, false);
        assert_eq!(nvic.interrupt_enable, 0x11);
    }

    #[test]
    fn test_set_interrupt() {
        let mut nvic = Nvic::new();

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::VBLANK, true);
        assert_eq!(nvic.interrupt_enable, 0x01);
        nvic.enable_interrupt(InterruptSources::LCD_STAT, true);
        assert_eq!(nvic.interrupt_enable, 0x03);

        nvic.set_interrupt(InterruptSources::SERIAL);
        let mut interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(false),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(true)
        }

        nvic.set_interrupt(InterruptSources::LCD_STAT);
        interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(true),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(false)
        }

        interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(false),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(true)
        }

        nvic.set_interrupt(InterruptSources::LCD_STAT);
        nvic.set_interrupt(InterruptSources::VBLANK);
        interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(true),
            Some(InterruptSources::LCD_STAT) => assert!(false),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(false)
        }

        interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(true),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(false)
        }

        interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(false),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(true)
        }
    }

    #[test]
    fn test_last_interrupt() {
        let mut nvic = Nvic::new();

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::VBLANK, true);
        assert_eq!(nvic.interrupt_enable, 0x01);
        nvic.enable_interrupt(InterruptSources::JOYPAD, true);
        assert_eq!(nvic.interrupt_enable, 0x11);

        nvic.set_interrupt(InterruptSources::JOYPAD);
        let mut interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(false),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(true),
            None => assert!(false)
        }

        interrupt = nvic.get_interrupt();
        match interrupt {
            Some(InterruptSources::VBLANK) => assert!(false),
            Some(InterruptSources::LCD_STAT) => assert!(false),
            Some(InterruptSources::TIMER) => assert!(false),
            Some(InterruptSources::SERIAL) => assert!(false),
            Some(InterruptSources::JOYPAD) => assert!(false),
            None => assert!(true)
        }
    }


    #[test]
    fn test_enable_it_from_byte() {
        let mut nvic = Nvic::new();

        nvic.from_byte(0b00001100);
        assert_eq!(nvic.to_byte(), 0b11101100);
    }
}