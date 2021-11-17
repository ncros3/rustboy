pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
const VRAM_SIZE: u16 = VRAM_END - VRAM_BEGIN + 1;

const TILE_LENGHT: u8 = 8;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PixelColor {
    WHITE,
    LIGHT_GRAY,
    DARK_GRAY,
    BLACK,
}

type Tile = [[PixelColor; 8]; 8];

fn create_tile() -> Tile {
    [[PixelColor::WHITE; 8]; 8]
}

pub struct Gpu {
    vram: [u8; VRAM_SIZE as usize],
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            vram: [0x00; VRAM_SIZE as usize],
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, data: u8) {
        self.vram[address as usize] = data;
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;

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
