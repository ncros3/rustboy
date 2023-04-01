use crate::cartridge::{MbcType, RomSize, RamSize, Mbc};

const RAM_ENABLE_SPACE_START: u16 = 0x0000;
const RAM_ENABLE_SPACE_END: u16 = 0x1FFF;

const ROM_BANK_NB_SPACE_START: u16 = 0x2000;
const ROM_BANK_NB_SPACE_END: u16 = 0x3FFF;

const RAM_BANK_NB_SPACE_START: u16 = 0x4000;
const RAM_BANK_NB_SPACE_END: u16 = 0x5FFF;

const BANKING_MODE_SPACE_START: u16 = 0x6000;
const BANKING_MODE_SPACE_END: u16 = 0x7FFF;

const ENABLE_RAM_FLAG: u8 = 0x0A;

const GB_ADDR_BIT_MASK: usize = 0x3FFF;
const ROM_BANK_BIT_OFFSET: usize = 14;
const RAM_BANK_BIT_OFFSET: usize = 19;

#[allow(non_camel_case_types)]
pub enum RomBankMask {
    MASK_1_BIT = 0x01,
    MASK_2_BIT = 0x03,
    MASK_3_BIT = 0x07,
    MASK_4_BIT = 0x0F,
    MASK_5_BIT = 0x1F,
}

pub struct Mbc1 {
    // config
    mbc_type: MbcType,
    rom_size: RomSize,
    ram_size: RamSize,
    // internal registers
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    banking_mode: bool,
    // memory
    rom_bank: Vec<u8>,
    ram_bank: Vec<u8>,
}

impl Mbc1 {
    pub fn new(mbc_type: MbcType, rom_size: RomSize, ram_size: RamSize, rom: &[u8]) -> Mbc1 {
        let mut rom_bank: Vec<u8> = vec![0xFF; rom_size.clone() as usize];
        let ram_bank: Vec<u8> = vec![0xFF; ram_size.clone() as usize];

        // copy all rom data
        for rom_index in 0..(rom_size as usize){
            rom_bank[rom_index as usize] = rom[rom_index as usize];
        }

        Mbc1 {
            // config
            mbc_type: mbc_type,
            rom_size: rom_size,
            ram_size: ram_size,
            // internal registers
            ram_enable: false,
            rom_bank_number: 1,
            ram_bank_number: 0,
            banking_mode: false,
            // memory
            rom_bank: rom_bank,
            ram_bank: ram_bank,
        }
    }
}

impl Mbc for Mbc1 {
    fn read_bank_0 (&self, address: usize) -> u8 {
        if self.banking_mode {
            let gb_addr = ((self.ram_bank_number as usize) << RAM_BANK_BIT_OFFSET) | (address & GB_ADDR_BIT_MASK);
            self.rom_bank[gb_addr]
        } else {
            let gb_addr = address & GB_ADDR_BIT_MASK;
            self.rom_bank[gb_addr]
        }
    }

    fn read_bank_n (&self, address: usize) -> u8 {
        let gb_addr = ((self.ram_bank_number as usize) << RAM_BANK_BIT_OFFSET) 
                            | ((self.rom_bank_number as usize) << ROM_BANK_BIT_OFFSET)
                            | (address & GB_ADDR_BIT_MASK);
        self.rom_bank[gb_addr]
    }

    fn read_ram (&self, address: usize) -> u8 {
        if self.ram_enable {
            if self.banking_mode {
                let gb_addr = address & 0x1FFF;
                self.ram_bank[gb_addr]
            } else {
                let gb_addr = ((self.ram_bank_number as usize) << 13)
                                    | (address & 0x1FFF);
                self.ram_bank[gb_addr]
            }
        } else {
            // RAM is disabled, returns 0xFF
            0xFF
        }
    }

    fn write_bank_0 (&mut self, address: usize, data: u8) {
        match address as u16 {
            RAM_ENABLE_SPACE_START..=RAM_ENABLE_SPACE_END => {
                if data == ENABLE_RAM_FLAG {
                    self.ram_enable = true;
                }
            },
            ROM_BANK_NB_SPACE_START..=ROM_BANK_NB_SPACE_END => {
                let rom_bank_mask = match self.rom_size {
                    RomSize::SIZE_32_KB => RomBankMask::MASK_1_BIT,
                    RomSize::SIZE_64_KB => RomBankMask::MASK_2_BIT,
                    RomSize::SIZE_128_KB => RomBankMask::MASK_3_BIT,
                    RomSize::SIZE_256_KB => RomBankMask::MASK_4_BIT,
                    _ => RomBankMask::MASK_5_BIT,
                };

                self.rom_bank_number = if data != 0 {
                    data & (rom_bank_mask as u8)
                } else {
                    // if register is set to 0, set it to 1 
                    1
                };
            },
            _ => panic!("mbc 1 bank 0 address {:x} doesn't exists.", address),
        }
    }

    fn write_bank_n (&mut self, address: usize, data: u8) {
        match address as u16 {
            RAM_BANK_NB_SPACE_START..=RAM_BANK_NB_SPACE_END => {
                self.ram_bank_number = data & 0x03;
            },
            BANKING_MODE_SPACE_START..=BANKING_MODE_SPACE_END => {
                self.banking_mode = (data & 0x01) != 0;
            },
            _ => panic!("mbc 1 bank n address {:x} doesn't exists.", address),
        }
    }

    fn write_ram (&mut self, address: usize, data: u8) {
        if self.ram_enable {
            if self.banking_mode {
                let gb_addr = address & 0x1FFF;
                self.ram_bank[gb_addr] = data;
            } else {
                let gb_addr = ((self.ram_bank_number as usize) << 13)
                                    | (address & 0x1FFF);
                self.ram_bank[gb_addr] = data;
            }
        } else {
            // do nothing when ram is disabled
        }
    }

    // not used for this mbc, doesn't do anything
    fn run (&mut self, _: u8) {}
}