mod rom;

use rom::Rom;

pub const CARTRIDGE_TYPE_OFFSET: u16 = 0x147;
pub const CARTRIDGE_ROM_SIZE_OFFSET: u16 = 0x148;
pub const CARTRIDGE_RAM_SIZE_OFFSET: u16 = 0x149;

#[allow(non_camel_case_types)]
pub enum MbcType {
    ROM_ONLY,
    MBC_1,
    MBC_1_RAM,
    MBC_1_RAM_BAT,
    MBC_2,
    MBC_2_BAT,
    ROM_RAM,
    ROM_RAM_BAT,
    MMM01,
    MMM01_RAM,
    MMM01_RAM_BAT,
    MBC_3_TIM_BAT,
    MBC_3_TIM_RAM_BAT,
    MBC_3,
    MBC_3_RAM,
    MBC_3_RAM_BAT,
    MBC_5,
    MBC_5_RAM,
    MBC_5_RAM_BAT,
    MBC_5_RUMBLE,
    MBC_5_RUMBLE_RAM,
    MBC_5_RUMBLE_RAM_BAT,
    MBC_6,
    MBC_7,
    CAMERA,
    TAMA_5,
    HUC3,
    HUC1,
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
    SIZE_32_KB,
    SIZE_64_KB,
    SIZE_128_KB,
    SIZE_256_KB,
    SIZE_512_KB,
    SIZE_1_MB,
    SIZE_2_MB,
    SIZE_4_MB,
    SIZE_8_MB,
}

impl std::fmt::Display for RomSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rom_size = match &*self {
            RomSize::SIZE_32_KB => "SIZE_32_KB",
            RomSize::SIZE_64_KB => "SIZE_64_KB",
            RomSize::SIZE_128_KB => "SIZE_128_KB",
            RomSize::SIZE_256_KB => "SIZE_256_KB",
            RomSize::SIZE_512_KB => "SIZE_512_KB",
            RomSize::SIZE_1_MB => "SIZE_1_MB",
            RomSize::SIZE_2_MB => "SIZE_2_MB",
            RomSize::SIZE_4_MB => "SIZE_4_MB",
            RomSize::SIZE_8_MB => "SIZE_8_MB",
        };
        write!(f, "{}", rom_size)
    }
}

#[allow(non_camel_case_types)]
pub enum RamSize {
    NO_RAM = 0x00,
    SIZE_8_KB = 0x02,
    SIZE_32_KB = 0x03,
    SIZE_128_KB = 0x04,
    SIZE_64_KB = 0x05,
}

impl std::fmt::Display for RamSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ram_size = match &*self {
            RamSize::NO_RAM => "NO_RAM",
            RamSize::SIZE_8_KB => "SIZE_8_KB",
            RamSize::SIZE_32_KB => "SIZE_32_KB",
            RamSize::SIZE_128_KB => "SIZE_128_KB",
            RamSize::SIZE_64_KB => "SIZE_64_KB",
        };
        write!(f, "{}", ram_size)
    }
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
        let mbc_type = get_mbc_type(rom[CARTRIDGE_TYPE_OFFSET as usize]);
        let rom_size = get_rom_size(rom[CARTRIDGE_ROM_SIZE_OFFSET as usize]);
        let ram_size = get_ram_size(rom[CARTRIDGE_RAM_SIZE_OFFSET as usize]);

        println!("Catridge with mbc type {}, rom size: {}, ram_size: {}", mbc_type, rom_size, ram_size);

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

fn get_mbc_type(raw_data: u8) -> MbcType {
    match raw_data {
        0x00 => MbcType::ROM_ONLY,
        0x01 => MbcType::MBC_1,
        0x02 => MbcType::MBC_1_RAM,
        0x03 => MbcType::MBC_1_RAM_BAT,
        0x05 => MbcType::MBC_2,
        0x06 => MbcType::MBC_2_BAT,
        0x08 => MbcType::ROM_RAM,
        0x09 => MbcType::ROM_RAM_BAT,
        0x0B => MbcType::MMM01,
        0x0C => MbcType::MMM01_RAM,
        0x0D => MbcType::MMM01_RAM_BAT,
        0x0F => MbcType::MBC_3_TIM_BAT,
        0x10 => MbcType::MBC_3_TIM_RAM_BAT,
        0x11 => MbcType::MBC_3,
        0x12 => MbcType::MBC_3_RAM,
        0x13 => MbcType::MBC_3_RAM_BAT,
        0x19 => MbcType::MBC_5,
        0x1A => MbcType::MBC_5_RAM,
        0x1B => MbcType::MBC_5_RAM_BAT,
        0x1C => MbcType::MBC_5_RUMBLE,
        0x1D => MbcType::MBC_5_RUMBLE_RAM,
        0x1E => MbcType::MBC_5_RUMBLE_RAM_BAT,
        0x20 => MbcType::MBC_6,
        0x22 => MbcType::MBC_7,
        0xFC => MbcType::CAMERA,
        0xFD => MbcType::TAMA_5,
        0xFE => MbcType::HUC3,
        0xFF => MbcType::HUC1,
        _=> panic!("Catridge with mbc type {:x} is unknown", raw_data),
    }
}

fn get_rom_size(raw_data: u8) -> RomSize {
    match raw_data {
        0x00 => RomSize::SIZE_32_KB,
        0x01 => RomSize::SIZE_64_KB,
        0x02 => RomSize::SIZE_128_KB,
        0x03 => RomSize::SIZE_256_KB,
        0x04 => RomSize::SIZE_512_KB,
        0x05 => RomSize::SIZE_1_MB,
        0x06 => RomSize::SIZE_2_MB,
        0x07 => RomSize::SIZE_4_MB,
        0x08 => RomSize::SIZE_8_MB,
        _=> panic!("Catridge with Rom size code {:x} is unknown", raw_data),
    }
}

fn get_ram_size(raw_data: u8) -> RamSize {
    match raw_data {
        0x00 => RamSize::NO_RAM,
        0x02 => RamSize::SIZE_8_KB,
        0x03 => RamSize::SIZE_32_KB,
        0x04 => RamSize::SIZE_128_KB,
        0x05 => RamSize::SIZE_64_KB,
        _=> panic!("Catridge with Ram size code {:x} is unknown", raw_data),
    }
}