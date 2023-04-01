use crate::cartridge::{MbcType, RomSize, RamSize, Mbc};
use crate::emulator::ONE_SECOND_IN_CYCLES;

const RAM_ENABLE_SPACE_START: u16 = 0x0000;
const RAM_ENABLE_SPACE_END: u16 = 0x1FFF;

const ROM_BANK_NB_SPACE_START: u16 = 0x2000;
const ROM_BANK_NB_SPACE_END: u16 = 0x3FFF;

const RAM_BANK_NB_SPACE_START: u16 = 0x4000;
const RAM_BANK_NB_SPACE_END: u16 = 0x5FFF;

const LATCH_CLOCK_SPACE_START: u16 = 0x6000;
const LATCH_CLOCK_SPACE_END: u16 = 0x7FFF;

const ENABLE_RAM_FLAG: u8 = 0x0A;

const GB_ADDR_BIT_MASK: usize = 0x3FFF;
const ROM_BANK_BIT_OFFSET: usize = 14;
const RAM_BANK_BIT_OFFSET: usize = 13;

#[allow(non_camel_case_types)]
enum RomBankMask {
    MASK_1_BIT = 0x01,
    MASK_2_BIT = 0x03,
    MASK_3_BIT = 0x07,
    MASK_4_BIT = 0x0F,
    MASK_5_BIT = 0x1F,
    MASK_6_BIT = 0x3F,
    MASK_7_BIT = 0x7F,
}

pub struct Mbc3 {
    // config
    mbc_type: MbcType,
    rom_size: RomSize,
    ram_size: RamSize,
    // internal registers
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    // memory
    rom_bank: Vec<u8>,
    ram_bank: Vec<u8>,
    // rtc
    latch_rtc_flag: bool,
    latch_rtc_enable: bool,
    rtc_cycles: usize,
    rtc_sec: u8,
    rtc_min: u8,
    rtc_hours: u8,
    rtc_day_lo: u8,
    rtc_day_hi: bool,
    rtc_halt: bool,
    rtc_overflow: bool,
    rtc_sec_latch: u8,
    rtc_min_latch: u8,
    rtc_hours_latch: u8,
    rtc_day_latch: u8,
}

impl Mbc3 {
    pub fn new(mbc_type: MbcType, rom_size: RomSize, ram_size: RamSize, rom: &[u8]) -> Mbc3 {
        let mut rom_bank: Vec<u8> = vec![0xFF; rom_size.clone() as usize];
        let ram_bank: Vec<u8> = vec![0xFF; ram_size.clone() as usize];

        // copy all rom data
        for rom_index in 0..(rom_size as usize){
            rom_bank[rom_index as usize] = rom[rom_index as usize];
        }

        Mbc3 {
            // config
            mbc_type: mbc_type,
            rom_size: rom_size,
            ram_size: ram_size,
            // internal registers
            ram_enable: false,
            rom_bank_number: 1,
            ram_bank_number: 0,
            // memory
            rom_bank: rom_bank,
            ram_bank: ram_bank,
            // rtc
            latch_rtc_flag: false,
            latch_rtc_enable: false,
            rtc_cycles: 0,
            rtc_sec: 0,
            rtc_min: 0,
            rtc_hours: 0,
            rtc_day_lo: 0,
            rtc_day_hi: false,
            rtc_halt: false,
            rtc_overflow: false,
            rtc_sec_latch: 0,
            rtc_min_latch: 0,
            rtc_hours_latch: 0,
            rtc_day_latch: 0,
        }
    }
}

impl Mbc for Mbc3 {
    fn read_bank_0 (&self, address: usize) -> u8 {
        let gb_addr = address & GB_ADDR_BIT_MASK;
        self.rom_bank[gb_addr]
    }

    fn read_bank_n (&self, address: usize) -> u8 {
        let gb_addr = ((self.rom_bank_number as usize) << ROM_BANK_BIT_OFFSET)
                            | (address & GB_ADDR_BIT_MASK);
        self.rom_bank[gb_addr]
    }

    fn read_ram (&self, address: usize) -> u8 {
        if self.ram_enable {
            match self.ram_bank_number {
                // here we access the ram banks
                0x00..=0x03 => {
                    let gb_addr = ((self.ram_bank_number as usize) << RAM_BANK_BIT_OFFSET)
                                | (address & 0x1FFF);
                    self.ram_bank[gb_addr]
                }
                // here we access rtc registers
                0x08 => self.rtc_sec_latch,
                0x09 => self.rtc_min_latch,
                0x0A => self.rtc_hours_latch,
                0x0B => self.rtc_day_latch,
                0x0C => (self.rtc_day_hi as u8)
                        | (self.rtc_halt as u8) << 6
                        | (self.rtc_overflow as u8) << 7,
                _ => 0xFF,
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
                    RomSize::SIZE_512_KB => RomBankMask::MASK_5_BIT,
                    RomSize::SIZE_1_MB => RomBankMask::MASK_6_BIT,
                    _ => RomBankMask::MASK_7_BIT,
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
                match data {
                    0x00..=0x03 => self.ram_bank_number = data & 0x03,
                    0x08..=0x0C => self.ram_bank_number = data,
                    _ => {/* do nothing here */},
                }
            },
            LATCH_CLOCK_SPACE_START..=LATCH_CLOCK_SPACE_END => {
                if data == 0x00 {
                    self.latch_rtc_flag = true;
                }

                if data == 0x01 && self.latch_rtc_flag {
                    self.latch_rtc_flag = false;
                    self.latch_rtc_enable = true;
                }
            },
            _ => panic!("mbc 1 bank n address {:x} doesn't exists.", address),
        }
    }

    fn write_ram (&mut self, address: usize, data: u8) {
        if self.ram_enable {
            match self.ram_bank_number {
                // here we access the ram banks
                0x00..=0x03 => {
                    let gb_addr = ((self.ram_bank_number as usize) << RAM_BANK_BIT_OFFSET)
                                | (address & 0x1FFF);
                    self.ram_bank[gb_addr] = data;
                }
                // here we access rtc registers
                0x08 => { self.rtc_sec = data }
                0x09 => { self.rtc_min = data }
                0x0A => { self.rtc_hours = data }
                0x0B => { self.rtc_day_lo = data }
                0x0C => { 
                    self.rtc_day_hi = (data & 0x01) != 0;
                    self.rtc_halt = (data & 0x40) != 0;
                    self.rtc_overflow = (data & 0x80) != 0;
                }
                _ => {/* do nothing here */}
            }
        } else {
            // do nothing when ram is disabled
        }
    }

    fn run (&mut self, cycles: u8) {
        if !self.rtc_halt {
            self.rtc_cycles += cycles as usize;

            if self.rtc_cycles > ONE_SECOND_IN_CYCLES {
                let add_sec = (self.rtc_cycles / ONE_SECOND_IN_CYCLES) as u8;
                // update rtc cycles
                self.rtc_cycles = self.rtc_cycles % ONE_SECOND_IN_CYCLES;
                // update rtc seconds
                self.rtc_sec +=  add_sec;
                if self.rtc_sec > 60 {
                    self.rtc_sec = 0;
                    self.rtc_min += 1;
                };
                // update rtc minutes
                if self.rtc_min > 60 {
                    self.rtc_min = 0;
                    self.rtc_hours += 1;
                }
                // update rtc hours
                if self.rtc_hours >= 24 {
                    self.rtc_hours = 0;
                    // check if day has overflowed
                    if self.rtc_day_hi && self.rtc_day_lo == 0xFF {
                        self.rtc_overflow = true;
                    }
                    // update day value
                    let (new_value, overflow) = self.rtc_day_lo.overflowing_add(1);
                    self.rtc_day_lo = new_value;
                    if overflow {self.rtc_day_hi = overflow};
                }
            }
        }

        if self.latch_rtc_enable {
            // save current counter
            self.rtc_sec_latch = self.rtc_sec;
            self.rtc_min_latch = self.rtc_min;
            self.rtc_hours_latch = self.rtc_hours;
            self.rtc_day_latch = self.rtc_day_lo;
            // reset latch
            self.latch_rtc_enable = false;
        }
    }
}