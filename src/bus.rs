pub struct Bus {
    memory: [u8; 0xFFFF],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            memory: [0x00; 0xFFFF],
        }
    }

    pub fn read_bus(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_bus(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }
}

#[cfg(test)]
mod bus_tests {
    use super::*;

    #[test]
    fn test_read_write_bus() {
        let mut bus = Bus::new();
        bus.write_bus(0x0001, 0xAA);
        bus.write_bus(0x0002, 0x55);
        bus.write_bus(0x0010, 0xAA);
        assert_eq!(bus.read_bus(0x0001), 0xAA);
        assert_eq!(bus.read_bus(0x0002), 0x55);
        assert_eq!(bus.read_bus(0x0010), 0xAA);
    }
}
