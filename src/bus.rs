use crate::gpu::{
    Gpu, 
    GpuInterruptRequest, 
    TileMapArea,
    ObjectSize};
use crate::nvic::{Nvic, InterruptSources};
use crate::timer::{Timer, Frequency};

pub const BOOT_ROM_BEGIN: u16 = 0x0000;
pub const BOOT_ROM_END: u16 = 0x00FF;
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
    boot_rom: Option<[u8; BOOT_ROM_SIZE as usize]>,
    rom_bank_0: [u8; ROM_BANK_0_SIZE as usize],
    rom_bank_n: [u8; ROM_BANK_N_SIZE as usize],
    external_ram: [u8; EXTERNAL_RAM_SIZE as usize],
    working_ram: [u8; WORKING_RAM_SIZE as usize],
    zero_page: [u8; ZERO_PAGE_SIZE as usize],
    pub gpu: Gpu,
    pub nvic: Nvic,
    timer: Timer,
    divider: Timer,
}

impl Bus {
    pub fn new() -> Bus {
        // initialize divider timer
        let mut divider = Timer::new(Frequency::F16384);
        divider.on = true;

        Bus {
            boot_rom: None,
            rom_bank_0: [0x00; ROM_BANK_0_SIZE as usize],
            rom_bank_n: [0x00; ROM_BANK_N_SIZE as usize],
            external_ram: [0x00; EXTERNAL_RAM_SIZE as usize],
            working_ram: [0x00; WORKING_RAM_SIZE as usize],
            zero_page: [0x00; ZERO_PAGE_SIZE as usize],
            gpu: Gpu::new(),
            nvic: Nvic::new(),
            timer: Timer::new(Frequency::F4096),
            divider: divider,
        }
    }

    pub fn load(&mut self, boot_rom: &[u8]){
        let mut rom_data = [0x00; BOOT_ROM_SIZE as usize];
        rom_data.copy_from_slice(boot_rom);
        self.boot_rom = Some(rom_data);
    }

    pub fn run(&mut self, runned_cycles: u8) {
        // run the system timer
        if self.timer.run(runned_cycles) {
            self.nvic.set_interrupt(InterruptSources::TIMER);
        }
        self.divider.run(runned_cycles);

        // run the GPU and catch interrupt requests if any
        self.gpu.run(runned_cycles);
    }

    pub fn read_bus(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => {
                match address {
                    BOOT_ROM_BEGIN..=BOOT_ROM_END => 
                        if let Some(boot_rom) = self.boot_rom {
                            boot_rom[address as usize]
                        } else {
                            self.rom_bank_0[address as usize]
                        }
                    _ => self.rom_bank_0[address as usize]
                }
            }
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => self.rom_bank_n[(address - ROM_BANK_N_BEGIN) as usize],
            
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            EXTERNAL_RAM_BEGIN..=EXTERNAL_RAM_END => {
                self.external_ram[(address - EXTERNAL_RAM_BEGIN) as usize]
            }
            WORKING_RAM_BEGIN..=WORKING_RAM_END => self.working_ram[(address - WORKING_RAM_BEGIN) as usize],
            ECHO_RAM_BEGIN..=ECHO_RAM_END => self.working_ram[(address - ECHO_RAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.gpu.read_oam((address - OAM_BEGIN) as usize),
            IO_REGISTERS_BEGIN..=IO_REGISTERS_END => self.read_io_register(address as usize),
            UNUSED_BEGIN..=UNUSED_END => 0, // unused memory
            ZERO_PAGE_BEGIN..=ZERO_PAGE_END => self.zero_page[(address - ZERO_PAGE_BEGIN) as usize],
            INTERRUPT_ENABLE_REGISTER => self.nvic.to_byte(),
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
                self.gpu.write_oam((address - OAM_BEGIN) as usize, data);
            }
            IO_REGISTERS_BEGIN..=IO_REGISTERS_END => self.write_io_register(address as usize, data),
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

    fn read_io_register(&self, address: usize) -> u8 {
        match address {
            0xFF00 => 0, // TODO: joypad
            0xFF01 => 0, // TODO: serial
            0xFF02 => 0, // TODO: serial
            0xFF04 => self.divider.value,
            0xFF0F => self.nvic.to_byte(),
            0xFF40 => {
                //TODO LCD Control
                0
            }
            0xFF41 => {
                //TODO LCD Controller Status
                0
            }
            0xFF42 => {
                //TODO Scroll Y Position
                0
            }
            0xFF44 => {
                //TODO Current Line
                0
            }
            _ => panic!("Reading from an unknown I/O register {:x}", address),
        }
    }

    fn write_io_register(&mut self, address: usize, data: u8) {
        match address {
            0xFF00 => { /* Joyad control */ }
            0xFF01 => { /* Serial Transfer */ }
            0xFF02 => { /* Serial Transfer Control */ }
            0xFF04 => self.divider.value = 0,
            0xFF05 => {
                self.timer.value = data;
            }
            0xFF06 => {
                self.timer.modulo = data;
            }
            0xFF07 => {
                self.timer.frequency = match data & 0b11 {
                    0b00 => Frequency::F4096,
                    0b11 => Frequency::F16384,
                    0b10 => Frequency::F65536,
                    _ => Frequency::F262144,
                };
                self.timer.on = (data & 0b100) == 0b100
            }
            0xFF0F => self.nvic.from_byte(data),
            0xFF10 => { /* Channel 1 Sweep register */ }
            0xFF11 => { /* Channel 1 Sound Length and Wave */ }
            0xFF12 => { /* Channel 1 Sound Control */ }
            0xFF13 => { /* Channel 1 Frequency lo */ }
            0xFF14 => { /* Channel 1 Control */ }
            0xFF16 => { /* Channel 2 Sound Control */ }
            0xFF17 => { /* Channel 2 Sound Control */ }
            0xFF18 => { /* Channel 2 Sound Control */ }
            0xFF19 => { /* Channel 2 Frequency hi data*/ }
            0xFF1A => { /* Channel 3 Sound on/off */ }
            0xFF1B => { /* Channel 3 Sound on/off */ }
            0xFF1C => { /* Channel 3 Sound on/off */ }
            0xFF1D => { /* Channel 3 Sound on/off */ }
            0xFF1E => { /* Channel 3 Sound on/off */ }
            0xFF20 => { /* Channel 4 Volumn */ }
            0xFF21 => { /* Channel 4 Volumn */ }
            0xFF22 => { /* Channel 4 Volumn */ }
            0xFF23 => { /* Channel 4 Counter/consecutive */ }
            0xFF24 => { /* Sound  Volume */ }
            0xFF25 => { /* Sound output terminal selection */ }
            0xFF26 => { /* Sound on/off */ }
            0xff30 | 0xff31 | 0xff32 | 0xff33 | 0xff34 | 0xff35 | 0xff36 | 0xff37 | 0xff38
            | 0xff39 | 0xff3a | 0xff3b | 0xff3c | 0xff3d | 0xff3e | 0xff3f => {
                //Wave Pattern RAM

            }
            0xFF40 => {
                //TODO LCD Control
            }
            0xFF41 => {
                //TODO LCD Controller Status
            }
            0xFF42 => {
                //TODO Viewport Y Offset
            }
            0xFF43 => {
                //TODO Viewport X Offset
            }
            0xFF45 => {
                // TODO compare line
            }
            0xFF46 => {
                // TODO: account for the fact this takes 160 microseconds
                // TODO implement DMA
            }
            0xFF47 => {
                //TODO Background Colors Setting
            }
            0xFF48 => {
                //TODO: implement object palette color 0
            }
            0xFF49 => {
                //TODO: implement object palette color 1
            }
            0xFF4A => {
                //TODO implement window x
            }
            0xFF4B => {
                //TODO implement window y
            }
            0xFF50 => {
                // Unmap boot ROM
                self.boot_rom = None;
            }
            0xFF7f => {
                // Writing to here does nothing
            }
            _ => panic!(
                "Writting '0b{:b}' to an unknown I/O register {:x}",
                data, address
            ),
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
