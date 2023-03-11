use crate::cartridge::{MbcType, RomSize, RamSize};

const RAM_ENABLE_SPACE_START: u16 = 0x0000;
const RAM_ENABLE_SPACE_END: u16 = 0x1FFF;

const ROM_BANK_NB_SPACE_START: u16 = 0x2000;
const ROM_BANK_NB_SPACE_END: u16 = 0x3FFF;

const RAM_BANK_NB_SPACE_START: u16 = 0x4000;
const RAM_BANK_NB_SPACE_END: u16 = 0x5FFF;

const BANKING_MODE_SPACE_START: u16 = 0x6000;
const BANKING_MODE_SPACE_END: u16 = 0x7FFF;

const ENABLE_RAM_FLAG: u8 = 0x0A;

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
    rom_bank: &[u8],
    ram_bank: &[u8],
}

impl Mbc1 {
    pub fn new(mbc_type: MbcType, rom_size: RomSize, ram_size: RamSize) -> Mbc1 {
        Mbc1 {
            // config
            mbc_type: mbc_type,
            rom_size: rom_size,
            ram_size: ram_size,
            // internal registers
            ram_enable: false,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: false,
            // memory
            rom_bank: &[u8],
            ram_bank: &[u8],
        }
    }

    pub fn write(&self, address: u16, data: u8) {
        match address {
            RAM_ENABLE_SPACE_START..=RAM_ENABLE_SPACE_END => {
                if data == ENABLE_RAM_FLAG {
                    self.ram_enable = true;
                }
            },
            ROM_BANK_NB_SPACE_END..=ROM_BANK_NB_SPACE_END => {
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
            RAM_BANK_NB_SPACE_END..=RAM_BANK_NB_SPACE_END => {
                self.ram_bank_number = data & 0x03;
            },
            BANKING_MODE_SPACE_START..=BANKING_MODE_SPACE_END => {
                self.banking_mode = (data & 0x01) != 0;
            },
        }
    }

    pub fn decode_addr(address: u16) -> u32 {

    }

    pub fn read_bank_0(&self, address: u16) -> u8 {
        if self.banking_mode {
            let gb_addr = (self.ram_bank_number << 19) + (address & 0x3F);
            self.rom_bank[gb_addr]
        } else {
            let gb_addr = address & 0x3F;
            self.rom_bank[gb_addr]
        }
    }

    pub fn read_bank_n(&self, address: u16) -> u8 {
        
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if self.ram_enable {
            if self.banking_mode {

            } else {

            }
        } else {
            // dummy read
            0xFF
        }
    }
}