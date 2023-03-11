use crate::cartridge::Mbc;

use crate::soc::peripheral::{ROM_BANK_0_SIZE, ROM_BANK_0_BEGIN, ROM_BANK_N_BEGIN, ROM_BANK_N_SIZE};

pub struct Rom {
    rom_bank_0: [u8; ROM_BANK_0_SIZE as usize],
    rom_bank_n: [u8; ROM_BANK_N_SIZE as usize],
}

impl Rom {
    pub fn new(rom: &[u8]) -> Rom {
        // copy bank 0 data
        let mut rom_bank_0 = [0x00; ROM_BANK_0_SIZE as usize];
        for rom_index in 0..ROM_BANK_0_SIZE {
            rom_bank_0[rom_index as usize] = rom[(ROM_BANK_0_BEGIN + rom_index) as usize];
        }

        // copy bank n data
        let mut rom_bank_n = [0x00; ROM_BANK_N_SIZE as usize];
        for rom_index in 0..ROM_BANK_N_SIZE {
            rom_bank_n[rom_index as usize] = rom[(ROM_BANK_N_BEGIN + rom_index) as usize];
        }

        Rom {
            rom_bank_0: rom_bank_0,
            rom_bank_n: rom_bank_n,
        }
    }
}

impl Mbc for Rom {
    fn read_bank_0 (&self, address: usize) -> u8 {
        self.rom_bank_0[address]
    }

    fn read_bank_n (&self, address: usize) -> u8 {
        self.rom_bank_n[address]
    }

    // not used for this mbc, returns 0xFF
    fn read_ram (&self, _: usize) -> u8 {
        0xFF
    }

    fn write_bank_0 (&mut self, address: usize, data: u8) {
        self.rom_bank_0[address] = data;
    }

    fn write_bank_n (&mut self, address: usize, data: u8) {
        self.rom_bank_n[address] = data;
    }

    // not used for this mbc, doesn't do anything
    fn write_ram (&mut self, _: usize, _: u8) {}
}