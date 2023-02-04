use crate::gpu::{
    Gpu, 
    GpuInterruptRequest, 
    TileMap,
    BackgroundAndWindowDataSelect,
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
        match self.gpu.run(runned_cycles) {
            GpuInterruptRequest::Both => {
                self.nvic.set_interrupt(InterruptSources::VBLANK);
                self.nvic.set_interrupt(InterruptSources::LCD_STAT);  
                },
            GpuInterruptRequest::VBlank => self.nvic.set_interrupt(InterruptSources::VBLANK),
            GpuInterruptRequest::LCDStat => self.nvic.set_interrupt(InterruptSources::LCD_STAT),
            GpuInterruptRequest::None => {},
        }
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
                // LCD Control
                (self.gpu.lcd_display_enabled as u8) << 7
                | ((self.gpu.window_tile_map == TileMap::X9C00) as u8) << 6
                | (self.gpu.window_display_enabled as u8) << 5
                | ((self.gpu.background_and_window_data_select
                    == BackgroundAndWindowDataSelect::X8000) as u8) << 4
                | ((self.gpu.background_tile_map == TileMap::X9C00) as u8) << 3
                | ((self.gpu.object_size == ObjectSize::OS8X16) as u8) << 2
                | (self.gpu.object_display_enabled as u8) << 1
                | (self.gpu.background_display_enabled as u8)
            }
            0xFF41 => {
                // LCD Controller Status
                let mode: u8 = self.gpu.mode.into();

                0b10000000
                | (self.gpu.line_equals_line_check_interrupt_enabled as u8) << 6
                | (self.gpu.oam_interrupt_enabled as u8) << 5
                | (self.gpu.vblank_interrupt_enabled as u8) << 4
                | (self.gpu.hblank_interrupt_enabled as u8) << 3
                | (self.gpu.line_equals_line_check as u8) << 2
                | mode
            }

            0xFF42 => {
                // Scroll Y Position
                self.gpu.viewport_y_offset
            }
            0xFF44 => {
                // Current Line
                self.gpu.line
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
                // LCD Control
                self.gpu.lcd_display_enabled = (data >> 7) == 1;
                self.gpu.window_tile_map = if ((data >> 6) & 0b1) == 1 {
                    TileMap::X9C00
                } else {
                    TileMap::X9800
                };
                self.gpu.window_display_enabled = ((data >> 5) & 0b1) == 1;
                self.gpu.background_and_window_data_select = if ((data >> 4) & 0b1) == 1 {
                    BackgroundAndWindowDataSelect::X8000
                } else {
                    BackgroundAndWindowDataSelect::X8800
                };
                self.gpu.background_tile_map = if ((data >> 3) & 0b1) == 1 {
                    TileMap::X9C00
                } else {
                    TileMap::X9800
                };
                self.gpu.object_size = if ((data >> 2) & 0b1) == 1 {
                    ObjectSize::OS8X16
                } else {
                    ObjectSize::OS8X8
                };
                self.gpu.object_display_enabled = ((data >> 1) & 0b1) == 1;
                self.gpu.background_display_enabled = (data & 0b1) == 1;
            }
            0xFF41 => {
                // LCD Controller Status
                self.gpu.line_equals_line_check_interrupt_enabled =
                    (data & 0b1000000) == 0b1000000;
                self.gpu.oam_interrupt_enabled = (data & 0b100000) == 0b100000;
                self.gpu.vblank_interrupt_enabled = (data & 0b10000) == 0b10000;
                self.gpu.hblank_interrupt_enabled = (data & 0b1000) == 0b1000;
            }
            0xFF42 => {
                // Viewport Y Offset
                self.gpu.viewport_y_offset = data;
            }
            0xFF43 => {
                // Viewport X Offset
                self.gpu.viewport_x_offset = data;
            }
            0xFF45 => {
                self.gpu.line_check = data;
            }
            0xFF46 => {
                // TODO: account for the fact this takes 160 microseconds
                let dma_source = (data as u16) << 8;
                let dma_destination = 0xFE00;
                for offset in 0..150 {
                    self.write_bus(
                        dma_destination + offset,
                        self.read_bus(dma_source + offset),
                    )
                }
            }
            0xFF47 => {
                // Background Colors Setting
                self.gpu.background_colors = data.into();
            }
            0xFF48 => {
                self.gpu.obj_0_color_3 = (data >> 6).into();
                self.gpu.obj_0_color_2 = ((data >> 4) & 0b11).into();
                self.gpu.obj_0_color_1 = ((data >> 2) & 0b11).into();
            }
            0xFF49 => {
                self.gpu.obj_1_color_3 = (data >> 6).into();
                self.gpu.obj_1_color_2 = ((data >> 4) & 0b11).into();
                self.gpu.obj_1_color_1 = ((data >> 2) & 0b11).into();
            }
            0xFF4A => {
                self.gpu.window.y = data;
            }
            0xFF4B => {
                self.gpu.window.x = data;
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
