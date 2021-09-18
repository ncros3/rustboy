pub struct Bus {
    memory: [u8; 0xFFFF],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            memory: [0x00; 0xFFFF],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }
}
