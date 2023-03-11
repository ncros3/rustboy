mod rom;

use rom::Rom;

#[allow(non_camel_case_types)]
pub enum MbcType {
    ROM_ONLY = 0x00,
    MBC_1 = 0x01,
    MBC_1_RAM = 0x02,
    MBC_1_RAM_BAT = 0x03,
    MBC_2 = 0x05,
    MBC_2_BAT = 0x06,
    ROM_RAM = 0x08,
    ROM_RAM_BAT = 0x09,
    MMM01 = 0x0B,
    MMM01_RAM = 0x0C,
    MMM01_RAM_BAT = 0x0D,
    MBC_3_TIM_BAT = 0x0F,
    MBC_3_TIM_RAM_BAT = 0x10,
    MBC_3 = 0x11,
    MBC_3_RAM = 0x12,
    MBC_3_RAM_BAT = 0x13,
    MBC_5 = 0x19,
    MBC_5_RAM = 0x1A,
    MBC_5_RAM_BAT = 0x1B,
    MBC_5_RUMBLE = 0x1C,
    MBC_5_RUMBLE_RAM = 0x1D,
    MBC_5_RUMBLE_RAM_BAT = 0x1E,
    MBC_6 = 0x20,
    MBC_7 = 0x22,
    CAMERA = 0xFC,
    TAMA_5 = 0xFD,
    HUC3 = 0xFE,
    HUC1 = 0xFF,
}

impl std::fmt::Display for MbcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mbc_type = match &*self {
            MbcType::ROM_ONLY => "ROM_ONLY",
            MbcType::MBC_1 => "MBC_1",
            MbcType::MBC_1_RAM => "MBC_1_RAM",
            MbcType::MBC_1_RAM_BAT => "MBC_1_RAM_BAT",
            MbcType::MBC_2 => "MBC_2",
            MbcType::MBC_2_BAT => "MBC_2_BAT",
            MbcType::ROM_RAM => "ROM_RAM",
            MbcType::ROM_RAM_BAT => "ROM_RAM_BAT",
            MbcType::MMM01 => "MMM01",
            MbcType::MMM01_RAM => "MMM01_RAM",
            MbcType::MMM01_RAM_BAT => "MMM01_RAM_BAT",
            MbcType::MBC_3_TIM_BAT => "MBC_3_TIM_BAT",
            MbcType::MBC_3_TIM_RAM_BAT => "MBC_3_TIM_RAM_BAT",
            MbcType::MBC_3 => "MBC_3",
            MbcType::MBC_3_RAM => "MBC_3_RAM",
            MbcType::MBC_3_RAM_BAT => "MBC_3_RAM_BAT",
            MbcType::MBC_5 => "MBC_5",
            MbcType::MBC_5_RAM => "MBC_5_RAM",
            MbcType::MBC_5_RAM_BAT => "MBC_5_RAM_BAT",
            MbcType::MBC_5_RUMBLE => "MBC_5_RUMBLE",
            MbcType::MBC_5_RUMBLE_RAM => "MBC_5_RUMBLE_RAM",
            MbcType::MBC_5_RUMBLE_RAM_BAT => "MBC_5_RUMBLE_RAM_BAT",
            MbcType::MBC_6 => "MBC_6",
            MbcType::MBC_7 => "MBC_7",
            MbcType::CAMERA => "CAMERA",
            MbcType::TAMA_5 => "TAMA_5",
            MbcType::HUC3 => "HUC3",
            MbcType::HUC1 => "HUC1",
        };

        write!(f, "{}", mbc_type)
    }
}

#[allow(non_camel_case_types)]
pub enum RomSize {
    SIZE_32_KB = 0x00,
    SIZE_64_KB = 0x01,
    SIZE_128_KB = 0x02,
    SIZE_256_KB = 0x03,
    SIZE_512_KB = 0x04,
    SIZE_1_MB = 0x05,
    SIZE_2_MB = 0x06,
    SIZE_4_MB = 0x07,
    SIZE_8_MB = 0x08
}

#[allow(non_camel_case_types)]
pub enum RamSize {
    NO_RAM = 0x00,
    SIZE_8_KB = 0x02,
    SIZE_32_KB = 0x03,
    SIZE_128_KB = 0x04,
    SIZE_64_KB = 0x05,
}

pub trait Mbc {
    fn read_bank_0 (&self, address: usize) -> u8;

    fn read_bank_n (&self, address: usize) -> u8;

    fn read_ram (&self, address: usize) -> u8;

    fn write_bank_0 (&mut self, _: usize, _: u8);

    fn write_bank_n (&mut self, _: usize, _: u8);

    fn write_ram (&mut self, _: usize, _: u8);
}

pub struct Cartridge {
    mbc: Box<dyn Mbc>,    
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> Cartridge {
        // find the mbctype in the rom data
        let mbc_type = MbcType::ROM_ONLY;

        // find the correct mbc structure for the cartridge interface
        let mbc = match mbc_type {
            MbcType::ROM_ONLY => Rom::new(rom),
            _ => panic!("Catridge with mbc type {} is not supported", mbc_type),
        };

        // return initialized cartridge
        Cartridge {
            mbc: Box::new(mbc),
        }
    }

    pub fn read_bank_0(&self, address: usize) -> u8 {
        self.mbc.read_bank_0(address)
    }

    pub fn read_ram(&self, address: usize) -> u8 {
        self.mbc.read_ram(address)
    }

    pub fn read_bank_n(&self, address: usize) -> u8 {
        self.mbc.read_bank_n(address)
    }

    pub fn write_bank_0(&mut self, address: usize, data: u8) {
        self.mbc.write_bank_0(address, data);
    }

    pub fn write_bank_n(&mut self, address: usize, data: u8) {
        self.mbc.write_bank_n(address, data);
    }

    pub fn write_ram(&mut self, address: usize, data: u8) {
        self.mbc.write_ram(address, data);
    }
}

