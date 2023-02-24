use crate::soc::peripheral::BOOT_ROM_SIZE;

pub struct BootRom {
    rom: [u8; BOOT_ROM_SIZE as usize],
    enabled: bool,
}

impl BootRom {
    pub fn new() -> BootRom {
        BootRom {
            rom: [0xFF; BOOT_ROM_SIZE as usize],
            enabled: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    pub fn load(&mut self, boot_rom: &[u8]){
        let mut rom_data = [0x00; BOOT_ROM_SIZE as usize];
        rom_data.copy_from_slice(boot_rom);
        self.rom = rom_data;
        // enable memory once load is complete
        self.enabled = true;
    }

    pub fn set_state(&mut self, state: bool) {
        self.enabled = state;
    }

    pub fn get_state(&self) -> bool {
        self.enabled
    }
}