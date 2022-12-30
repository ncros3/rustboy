pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
const VRAM_SIZE: u16 = VRAM_END - VRAM_BEGIN + 1;

const TILE_LENGHT: u8 = 8;
const TILE_SET_SIZE: u16 = 384;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PixelColor {
    WHITE,
    LIGHT_GRAY,
    DARK_GRAY,
    BLACK,
}

type Tile = [[PixelColor; TILE_LENGHT as usize]; TILE_LENGHT as usize];

fn create_tile() -> Tile {
    [[PixelColor::WHITE; TILE_LENGHT as usize]; TILE_LENGHT as usize]
}

pub struct Gpu {
    vram: [u8; VRAM_SIZE as usize],
    tile_set: [Tile; TILE_SET_SIZE as usize],
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            vram: [0x00; VRAM_SIZE as usize],
            tile_set: [[[PixelColor::WHITE; TILE_LENGHT as usize]; TILE_LENGHT as usize]; TILE_SET_SIZE as usize],
        }
    }

    pub fn run(&mut self, runned_cycles: u8) {
        
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, data: u8) {
        self.vram[address as usize] = data;

        // check if address exceeds tile set storage
        if address >= 0x1800 {
            return
        }

        // save data in Tile set dedicated structure
        let normalized_address = (address & 0xFFFE) as usize;
        let byte1 = self.vram[normalized_address];
        let byte2 = self.vram[normalized_address + 1];

        let tile_index = (address / 16) as usize;
        let row_index = ((address % 16) / 2) as usize;

        for pixel_index in 0..8 {
            let mask = 1 << (7 - pixel_index);
            let lsb = byte1 & mask;
            let msb = byte2 & mask;

            let value = match (lsb != 0, msb != 0) {
                (true, true) => PixelColor::BLACK,
                (false, true) => PixelColor::DARK_GRAY,
                (true, false) => PixelColor::LIGHT_GRAY,
                (false, false) => PixelColor::WHITE,
            };

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;

    #[test]
    fn test_fill_tile_set() {
        let mut gpu = Gpu::new();
        gpu.write_vram(0x0000, 0xCC);
        gpu.write_vram(0x0001, 0xAA);

        assert_eq!(gpu.tile_set[0][0][0], PixelColor::BLACK);
        assert_eq!(gpu.tile_set[0][0][5], PixelColor::LIGHT_GRAY);
        assert_eq!(gpu.tile_set[0][0][2], PixelColor::DARK_GRAY);

        gpu.write_vram(0x00F0, 0xCC);
        gpu.write_vram(0x00F1, 0xAA);

        assert_eq!(gpu.tile_set[15][0][0], PixelColor::BLACK);
        assert_eq!(gpu.tile_set[15][0][5], PixelColor::LIGHT_GRAY);
        assert_eq!(gpu.tile_set[15][0][2], PixelColor::DARK_GRAY);
    }

    #[test]
    fn test_create_tile() {
        let mut new_tile = create_tile();
        assert_eq!(new_tile[1][1], PixelColor::WHITE);

        new_tile[1][2] = PixelColor::DARK_GRAY;
        assert_eq!(new_tile[1][2], PixelColor::DARK_GRAY);
    }

    #[test]
    fn test_read_write_vram() {
        let mut gpu = Gpu::new();
        gpu.write_vram(0x0001, 0xAA);
        gpu.write_vram(0x0002, 0x55);
        gpu.write_vram(0x0010, 0xAA);
        assert_eq!(gpu.read_vram(0x0001), 0xAA);
        assert_eq!(gpu.read_vram(0x0002), 0x55);
        assert_eq!(gpu.read_vram(0x0010), 0xAA);
    }
}
