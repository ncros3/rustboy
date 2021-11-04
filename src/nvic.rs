enum InterruptSources {
    VBLANK = 0,
    LCD_STAT = 1,
    TIMER = 2,
    SERIAL = 3,
    JOYPAD = 4,
}

pub struct Nvic {
    interrupt_master_enable: bool,
    interrupt_enable: u8,
    interrupt_flag; u8,
}

impl Nvic {
    pub fn new() -> Nvic {
        Nvic {
            interrupt_master_enable: 0,
            interrupt_enable: 0,
            interrupt_flag: 0,
        }
    }

    pub fn master_enable(&mut self, enable: bool) {
        self.interrupt_master_enable = enable;
    }

    pub fn enable_interrupt(&mut self, source: InterruptSources, enable: bool) {
        self.interrupt_enable = enable << source;
    }
}