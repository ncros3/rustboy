use crate::cartridge::Mbc;
use crate::soc::peripheral::{ROM_BANK_0_SIZE, ROM_BANK_N_SIZE};

pub struct Rom {
    rom_bank: [u8; (ROM_BANK_0_SIZE + ROM_BANK_N_SIZE) as usize],
}

impl Rom {
    pub fn new(rom: &[u8]) -> Rom {
        // copy  data
        let mut rom_bank = [0x00; (ROM_BANK_0_SIZE + ROM_BANK_N_SIZE) as usize];
        for rom_index in 0..(ROM_BANK_0_SIZE + ROM_BANK_N_SIZE) {
            rom_bank[rom_index as usize] = rom[rom_index as usize];
        }

        Rom {
            rom_bank : rom_bank,
        }
    }
}

impl Mbc for Rom {
    fn read_bank_0 (&self, address: usize) -> u8 {
        self.rom_bank[address as usize]
    }

    fn read_bank_n (&self, address: usize) -> u8 {
        self.rom_bank[address as usize]
    }

    // not used for this mbc, returns 0xFF
    fn read_ram (&self, _: usize) -> u8 {
        0xFF
    }

    fn write_bank_0 (&mut self, address: usize, data: u8) {
        self.rom_bank[address as usize] = data;
    }

    fn write_bank_n (&mut self, address: usize, data: u8) {
        self.rom_bank[address as usize] = data;
    }

    // not used for this mbc, doesn't do anything
    fn write_ram (&mut self, _: usize, _: u8) {}
}