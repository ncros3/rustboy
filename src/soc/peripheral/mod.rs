pub mod gpu;
pub mod nvic;
mod timer;
mod bootrom;

use gpu::Gpu;
use nvic::Nvic;
use timer::Timer;
use bootrom::BootRom;

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

pub struct Peripheral {
    boot_rom: BootRom,
    rom_bank_0: [u8; ROM_BANK_0_SIZE as usize],
    rom_bank_n: [u8; ROM_BANK_N_SIZE as usize],
    external_ram: [u8; EXTERNAL_RAM_SIZE as usize],
    working_ram: [u8; WORKING_RAM_SIZE as usize],
    zero_page: [u8; ZERO_PAGE_SIZE as usize],
    pub gpu: Gpu,
    pub nvic: Nvic,
    timer: Timer,
    // dma
    dma_cycles: u8,
    dma_start_adress: u16,
    dma_enabled: bool,
}

impl Peripheral {
    pub fn new() -> Peripheral {
        Peripheral {
            boot_rom: BootRom::new(),
            rom_bank_0: [0xFF; ROM_BANK_0_SIZE as usize],
            rom_bank_n: [0xFF; ROM_BANK_N_SIZE as usize],
            external_ram: [0xFF; EXTERNAL_RAM_SIZE as usize],
            working_ram: [0xFF; WORKING_RAM_SIZE as usize],
            zero_page: [0xFF; ZERO_PAGE_SIZE as usize],
            gpu: Gpu::new(),
            nvic: Nvic::new(),
            timer: Timer::new(),
            dma_cycles: 0,
            dma_start_adress: 0xFFFF,
            dma_enabled: false,
        }
    }

    pub fn run(&mut self, runned_cycles: u8) {
        // run the timer
        self.timer.run(runned_cycles, &mut self.nvic);

        // run the DMA
        if self.dma_enabled {
            // copy data
            for mem_index in 0..runned_cycles {
                if self.dma_cycles < OAM_SIZE as u8 {
                    let data = self.read(self.dma_start_adress + (self.dma_cycles + mem_index) as u16);
                    self.gpu.write_oam((mem_index + self.dma_cycles) as usize, data);
                }
            }
            // update internal timer
            self.dma_cycles += runned_cycles;
            // check if we reached the end of the dma transfert
            if self.dma_cycles >= OAM_SIZE as u8{
                // disable dma
                self.dma_enabled = false;
                self.dma_cycles = 0;
            }
        }

        // run the GPU 
        self.gpu.run(runned_cycles, &mut self.nvic);
    }

    pub fn load_bootrom(&mut self, boot_rom: &[u8]){
        self.boot_rom.load(boot_rom);
    }

    pub fn load_rom(&mut self, rom: &[u8]){
        // copy bank 0 data
        for rom_index in 0..ROM_BANK_0_SIZE {
            self.rom_bank_0[rom_index as usize] = rom[rom_index as usize];
        }
        // copy bank n data
        for rom_index in 0..ROM_BANK_N_SIZE {
            self.rom_bank_n[rom_index as usize] = rom[(ROM_BANK_N_BEGIN + rom_index) as usize];
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => {
                match address {
                    BOOT_ROM_BEGIN..=BOOT_ROM_END => 
                        if self.boot_rom.get_state() {
                            self.boot_rom.read(address)
                        } else {
                            self.rom_bank_0[address as usize]
                        }
                    _ => self.rom_bank_0[address as usize]
                }
            }
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => self.rom_bank_n[(address - ROM_BANK_N_BEGIN) as usize],
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram_ext(address - VRAM_BEGIN),
            EXTERNAL_RAM_BEGIN..=EXTERNAL_RAM_END => self.external_ram[(address - EXTERNAL_RAM_BEGIN) as usize],
            WORKING_RAM_BEGIN..=WORKING_RAM_END => self.working_ram[(address - WORKING_RAM_BEGIN) as usize],
            ECHO_RAM_BEGIN..=ECHO_RAM_END => self.working_ram[(address - ECHO_RAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.gpu.read_oam_ext((address - OAM_BEGIN) as usize),
            IO_REGISTERS_BEGIN..=IO_REGISTERS_END => self.read_io_register(address as usize),
            UNUSED_BEGIN..=UNUSED_END => 0, // unused memory
            ZERO_PAGE_BEGIN..=ZERO_PAGE_END => self.zero_page[(address - ZERO_PAGE_BEGIN) as usize],
            INTERRUPT_ENABLE_REGISTER => self.nvic.get_it_enable(),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => {
                self.rom_bank_0[address as usize] = data;
            }
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram_ext(address - VRAM_BEGIN, data),
            EXTERNAL_RAM_BEGIN..=EXTERNAL_RAM_END => {
                self.external_ram[(address - EXTERNAL_RAM_BEGIN) as usize] = data;
            }
            WORKING_RAM_BEGIN..=WORKING_RAM_END => {
                self.working_ram[(address - WORKING_RAM_BEGIN) as usize] = data;
            }
            OAM_BEGIN..=OAM_END => self.gpu.write_oam_ext((address - OAM_BEGIN) as usize, data),
            IO_REGISTERS_BEGIN..=IO_REGISTERS_END => self.write_io_register(address as usize, data),
            UNUSED_BEGIN..=UNUSED_END => { /* Writing to here does nothing */ }
            ZERO_PAGE_BEGIN..=ZERO_PAGE_END => {
                self.zero_page[(address - ZERO_PAGE_BEGIN) as usize] = data;
            }
            INTERRUPT_ENABLE_REGISTER => self.nvic.set_it_enable(data),
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
            0xFF04 => self.timer.get_divider(),
            0xFF05 => self.timer.get_value(),
            0xFF06 => self.timer.get_modulo(),
            0xFF0F => self.nvic.get_it_flag(),
            0xFF40 => self.gpu.control_to_byte(),
            0xFF41 => self.gpu.status_to_byte(),
            0xFF42 => self.gpu.get_scy(),
            0xFF43 => self.gpu.get_scx(),
            0xFF44 => self.gpu.get_current_line(),
            0xFF45 => self.gpu.get_compare_line(),
            _ => panic!("Reading from an unknown I/O register {:x}", address),
        }
    }

    fn write_io_register(&mut self, address: usize, data: u8) {
        match address {
            0xFF00 => { /* Joyad control */ }
            0xFF01 => { /* Serial Transfer */ }
            0xFF02 => { /* Serial Transfer Control */ }
            0xFF04 => self.timer.set_divider(),
            0xFF05 => self.timer.set_value(data),
            0xFF06 => self.timer.set_modulo(data),
            0xFF07 => self.timer.settings_from_byte(data),
            0xFF0F => self.nvic.set_it_flag(data),
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
            0xFF40 => self.gpu.control_from_byte(data),
            0xFF41 => self.gpu.status_from_byte(data),
            0xFF42 => self.gpu.set_scy(data),
            0xFF43 => self.gpu.set_scx(data),
            0xFF45 => self.gpu.set_compare_line(data),
            0xFF46 => {
                self.dma_start_adress = (data as u16) << 8;
                self.dma_enabled = true;
            }
            0xFF47 => self.gpu.set_background_palette(data),
            0xFF48 => self.gpu.set_object_palette_0(data),
            0xFF49 => self.gpu.set_object_palette_1(data),
            0xFF4A => self.gpu.set_window_y(data),
            0xFF4B => self.gpu.set_window_x(data),
            0xFF50 => self.boot_rom.set_state(false),
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
mod peripheral_tests {
    use super::*;

    #[test]
    fn test_read_write() {
        let mut peripheral = Peripheral::new();
        peripheral.write(0x0001, 0xAA);
        peripheral.write(0x0002, 0x55);
        peripheral.write(0x0010, 0xAA);
        assert_eq!(peripheral.read(0x0001), 0xAA);
        assert_eq!(peripheral.read(0x0002), 0x55);
        assert_eq!(peripheral.read(0x0010), 0xAA);
    }

    #[test]
    fn test_read_write_vram() {
        let mut peripheral = Peripheral::new();
        peripheral.write(0x0001 + VRAM_BEGIN, 0xAA);
        peripheral.write(0x0002 + VRAM_BEGIN, 0x55);
        peripheral.write(0x0010 + VRAM_BEGIN, 0xAA);
        assert_eq!(peripheral.read(0x0001 + VRAM_BEGIN), 0xAA);
        assert_eq!(peripheral.read(0x0002 + VRAM_BEGIN), 0x55);
        assert_eq!(peripheral.read(0x0010 + VRAM_BEGIN), 0xAA);
    }

    #[test]
    fn test_oam_dma() {
        let mut peripheral = Peripheral::new();
        let address = 0x1000;
        // init data
        peripheral.write(address, 0xAA);
        peripheral.write(address + 0x007F, 0xAA);
        peripheral.write(address + 0x009F, 0x55);

        // set dma
        peripheral.write(0xFF46, (0x1000 >> 8) as u8);

        // run peripheral for 160 cycles
        for _ in 0..OAM_SIZE {
            peripheral.run(1);
        }

        // check oam memory
        assert_eq!(peripheral.gpu.read_oam(0x00), 0xAA);
        assert_eq!(peripheral.gpu.read_oam(0x7F), 0xAA);
        assert_eq!(peripheral.gpu.read_oam(0x9F), 0x55);
    }
}