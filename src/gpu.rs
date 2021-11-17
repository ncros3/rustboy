const VRAM_BEGIN: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;
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
}
