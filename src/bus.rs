use crate::gpu::Gpu;
use crate::nvic::Nvic;

pub const BOOT_ROM_BEGIN: u16 = 0x00;
pub const BOOT_ROM_END: u16 = 0xFF;
pub const BOOT_ROM_SIZE: u16 = BOOT_ROM_END - BOOT_ROM_BEGIN + 1;

pub const ROM_BANK_0_BEGIN: u16 = 0x0000;
pub const ROM_BANK_0_END: u16 = 0x3FFF;
pub const ROM_BANK_0_SIZE: u16 = ROM_BANK_0_END - ROM_BANK_0_BEGIN + 1;

pub const ROM_BANK_N_BEGIN: u16 = 0x4000;
pub const ROM_BANK_N_END: u16 = 0x7FFF;
pub const ROM_BANK_N_SIZE: u16 = ROM_BANK_N_END - ROM_BANK_N_BEGIN + 1;

pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: u16 = VRAM_END - VRAM_BEGIN + 1;

pub const EXTERNAL_RAM_BEGIN: u16 = 0xA000;
pub const EXTERNAL_RAM_END: u16 = 0xBFFF;
pub const EXTERNAL_RAM_SIZE: u16 = EXTERNAL_RAM_END - EXTERNAL_RAM_BEGIN + 1;

pub const WORKING_RAM_BEGIN: u16 = 0xC000;
pub const WORKING_RAM_END: u16 = 0xDFFF;
pub const WORKING_RAM_SIZE: u16 = WORKING_RAM_END - WORKING_RAM_BEGIN + 1;

pub const ECHO_RAM_BEGIN: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;

pub const OAM_BEGIN: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OAM_SIZE: u16 = OAM_END - OAM_BEGIN + 1;

pub const UNUSED_BEGIN: u16 = 0xFEA0;
pub const UNUSED_END: u16 = 0xFEFF;

pub const IO_REGISTERS_BEGIN: u16 = 0xFF00;
pub const IO_REGISTERS_END: u16 = 0xFF7F;

pub const ZERO_PAGE_BEGIN: u16 = 0xFF80;
pub const ZERO_PAGE_END: u16 = 0xFFFE;
pub const ZERO_PAGE_SIZE: u16 = ZERO_PAGE_END - ZERO_PAGE_BEGIN + 1;

pub const INTERRUPT_ENABLE_REGISTER: u16 = 0xFFFF;

pub const VBLANK_VECTOR: u16 = 0x40;
pub const LCDSTAT_VECTOR: u16 = 0x48;
pub const TIMER_VECTOR: u16 = 0x50;

pub struct Bus {
    boot_rom: [u8; BOOT_ROM_SIZE as usize],
    rom_bank_0: [u8; ROM_BANK_0_SIZE as usize],
    rom_bank_n: [u8; ROM_BANK_N_SIZE as usize],
    external_ram: [u8; EXTERNAL_RAM_SIZE as usize],
    working_ram: [u8; WORKING_RAM_SIZE as usize],
    zero_page: [u8; ZERO_PAGE_SIZE as usize],
    gpu: Gpu,
    pub nvic: Nvic,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            boot_rom: [0x00; BOOT_ROM_SIZE as usize],
            rom_bank_0: [0x00; ROM_BANK_0_SIZE as usize],
            rom_bank_n: [0x00; ROM_BANK_N_SIZE as usize],
            external_ram: [0x00; EXTERNAL_RAM_SIZE as usize],
            working_ram: [0x00; WORKING_RAM_SIZE as usize],
            zero_page: [0x00; ZERO_PAGE_SIZE as usize],
            gpu: Gpu::new(),
            nvic: Nvic::new(),
        }
    }

    pub fn run(&mut self, runned_cycles: u8) {
        self.gpu.run(runned_cycles);
    }

    pub fn read_bus(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.rom_bank_0[address as usize],
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => self.rom_bank_n[(address - ROM_BANK_N_BEGIN) as usize],
            
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            EXTERNAL_RAM_BEGIN..=EXTERNAL_RAM_END => {
                self.external_ram[(address - EXTERNAL_RAM_BEGIN) as usize]
            }
            WORKING_RAM_BEGIN..=WORKING_RAM_END => self.working_ram[(address - WORKING_RAM_BEGIN) as usize],
            ECHO_RAM_BEGIN..=ECHO_RAM_END => self.working_ram[(address - ECHO_RAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => 0, //TODO: OAM memory
            IO_REGISTERS_BEGIN..=IO_REGISTERS_END => 0, //TODO: IO register
            UNUSED_BEGIN..=UNUSED_END => 0, // unused memory
            ZERO_PAGE_BEGIN..=ZERO_PAGE_END => self.zero_page[(address - ZERO_PAGE_BEGIN) as usize],
            INTERRUPT_ENABLE_REGISTER => self.nvic.to_byte(),
            _ => {
                panic!(
                    "Reading from an unkown part of memory at address 0x{:x}",
                    address
                );
            }
        }
    }

    pub fn write_bus(&mut self, address: u16, data: u8) {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => {
                self.rom_bank_0[address as usize] = data;
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, data),
            EXTERNAL_RAM_BEGIN..=EXTERNAL_RAM_END => {
                self.external_ram[(address - EXTERNAL_RAM_BEGIN) as usize] = data;
            }
            WORKING_RAM_BEGIN..=WORKING_RAM_END => {
                self.working_ram[(address - WORKING_RAM_BEGIN) as usize] = data;
            }
            OAM_BEGIN..=OAM_END => {
                //TODO: write to OAM
            }
            IO_REGISTERS_BEGIN..=IO_REGISTERS_END => {
                //TODO: write to IO registers
            }
            UNUSED_BEGIN..=UNUSED_END => { /* Writing to here does nothing */ }
            ZERO_PAGE_BEGIN..=ZERO_PAGE_END => {
                self.zero_page[(address - ZERO_PAGE_BEGIN) as usize] = data;
            }
            INTERRUPT_ENABLE_REGISTER => self.nvic.from_byte(data),
            _ => {
                panic!(
                    "Writing to an unkown part of memory at address 0x{:x}",
                    address
                );
            }
        }
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

    #[test]
    fn test_read_write_vram() {
        let mut bus = Bus::new();
        bus.write_bus(0x0001 + VRAM_BEGIN, 0xAA);
        bus.write_bus(0x0002 + VRAM_BEGIN, 0x55);
        bus.write_bus(0x0010 + VRAM_BEGIN, 0xAA);
        assert_eq!(bus.read_bus(0x0001 + VRAM_BEGIN), 0xAA);
        assert_eq!(bus.read_bus(0x0002 + VRAM_BEGIN), 0x55);
        assert_eq!(bus.read_bus(0x0010 + VRAM_BEGIN), 0xAA);
    }
}
