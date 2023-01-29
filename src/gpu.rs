use crate::bus::{VRAM_SIZE, OAM_SIZE};

const TILE_LENGHT: u8 = 8;
const TILE_SET_SIZE: u16 = 384;

const NUMBER_OF_SPRITES: usize = 40;
const SPRITE_LENGTH_IN_BYTE: usize = 4;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PixelColor {
    WHITE,
    LIGHT_GRAY,
    DARK_GRAY,
    BLACK,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ObjectPalette {
    Zero,
    One,
}

impl Default for ObjectPalette {
    fn default() -> Self {
        ObjectPalette::Zero
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ObjectData {
    x: i16,
    y: i16,
    tile: u8,
    palette: ObjectPalette,
    xflip: bool,
    yflip: bool,
    priority: bool,
}

impl Default for ObjectData {
    fn default() -> Self {
        ObjectData {
            x: -16,
            y: -8,
            tile: Default::default(),
            palette: Default::default(),
            xflip: Default::default(),
            yflip: Default::default(),
            priority: Default::default(),
        }
    }
}

type Tile = [[PixelColor; TILE_LENGHT as usize]; TILE_LENGHT as usize];

fn create_tile() -> Tile {
    [[PixelColor::WHITE; TILE_LENGHT as usize]; TILE_LENGHT as usize]
}

pub struct Gpu {
    // VRAM is a memory area used to store graphics such as backgrounds and sprites
    vram: [u8; VRAM_SIZE as usize],
    // tile set is a buffer computed by the GPU from VRAM at each write operation
    tile_set: [Tile; TILE_SET_SIZE as usize],
    // OAM is a memory area used to store sprites attributes
    // Sprites data are stored in VRAM memory $8000-8FFF
    oam: [u8; OAM_SIZE as usize],
    // object data is a buffer computed by the GPU from OAM at each write operation
    object_data: [ObjectData; NUMBER_OF_SPRITES]
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            vram: [0x00; VRAM_SIZE as usize],
            tile_set: [create_tile(); TILE_SET_SIZE as usize],
            oam: [0; OAM_SIZE as usize],
            object_data: [Default::default(); NUMBER_OF_SPRITES],
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

    pub fn write_oam(&mut self, index: usize, data: u8) {
        // save data in OAM memory
        self.oam[index] = data;

        // convert OAM raw data in structure OAM attributes
        let object_index = index / SPRITE_LENGTH_IN_BYTE;
        if object_index > NUMBER_OF_SPRITES {
            return;
        }

        let byte = index % SPRITE_LENGTH_IN_BYTE;

        // get a reference to the object
        let mut object_data = self.object_data.get_mut(object_index).unwrap();
        match byte {
            0 => object_data.y = (data as i16) - 0x10,
            1 => object_data.x = (data as i16) - 0x8,
            2 => object_data.tile = data,
            _ => {
                object_data.palette = if (data & 0x10) != 0 {
                    ObjectPalette::One
                } else {
                    ObjectPalette::Zero
                };
                object_data.xflip = (data & 0x20) != 0;
                object_data.yflip = (data & 0x40) != 0;
                object_data.priority = (data & 0x80) == 0;
            }
        }
    }

    pub fn read_oam(&self, address: usize) -> u8 {
        self.oam[address]
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
