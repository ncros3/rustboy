use crate::bus::{VRAM_BEGIN, VRAM_SIZE, OAM_SIZE};

const TILE_LENGHT: u8 = 8;
const TILE_SET_SIZE: u16 = 384;

const NUMBER_OF_SPRITES: usize = 40;
const SPRITE_LENGTH_IN_BYTE: usize = 4;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}
impl Default for TilePixelValue {
    fn default() -> Self {
        TilePixelValue::Zero
    }
}

type Tile = [[TilePixelValue; TILE_LENGHT as usize]; TILE_LENGHT as usize];

fn create_tile() -> Tile {
    [[TilePixelValue::Zero; TILE_LENGHT as usize]; TILE_LENGHT as usize]
}

#[derive(Copy, Clone)]
pub enum PixelColor {
    WHITE = 255,
    LIGHT_GRAY = 192,
    DARK_GRAY = 96,
    BLACK = 0,
}

impl std::convert::From<u8> for PixelColor {
    fn from(n: u8) -> Self {
        match n {
            0 => PixelColor::WHITE,
            1 => PixelColor::LIGHT_GRAY,
            2 => PixelColor::DARK_GRAY,
            3 => PixelColor::BLACK,
            _ => panic!("Cannot covert {} to color", n),
        }
    }
}

pub struct BackgroundColors(PixelColor, PixelColor, PixelColor, PixelColor);

impl BackgroundColors {
    fn new() -> BackgroundColors {
        BackgroundColors(
            PixelColor::WHITE,
            PixelColor::LIGHT_GRAY,
            PixelColor::DARK_GRAY,
            PixelColor::BLACK,
        )
    }
}

impl std::convert::From<u8> for BackgroundColors {
    fn from(value: u8) -> Self {
        BackgroundColors(
            (value & 0b11).into(),
            ((value >> 2) & 0b11).into(),
            ((value >> 4) & 0b11).into(),
            (value >> 6).into(),
        )
    }
}

#[derive(Eq, PartialEq)]
pub enum TileMap {
    X9800,
    X9C00,
}

#[derive(Eq, PartialEq)]
pub enum BackgroundAndWindowDataSelect {
    X8000,
    X8800,
}

#[derive(Eq, PartialEq)]
pub enum ObjectSize {
    OS8X8,
    OS8X16,
}

pub struct Window {
    pub x: u8,
    pub y: u8,
}

#[derive(Copy, Clone)]
pub enum Mode {
    HorizontalBlank,
    VerticalBlank,
    OAMAccess,
    VRAMAccess,
}

impl std::convert::From<Mode> for u8 {
    fn from(value: Mode) -> Self {
        match value {
            Mode::HorizontalBlank => 0,
            Mode::VerticalBlank => 1,
            Mode::OAMAccess => 2,
            Mode::VRAMAccess => 3,
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum InterruptRequest {
    None,
    VBlank,
    LCDStat,
    Both,
}

impl InterruptRequest {
    fn add(&mut self, other: InterruptRequest) {
        match self {
            InterruptRequest::None => *self = other,
            InterruptRequest::VBlank if other == InterruptRequest::LCDStat => {
                *self = InterruptRequest::Both
            }
            InterruptRequest::LCDStat if other == InterruptRequest::VBlank => {
                *self = InterruptRequest::Both
            }
            _ => {}
        };
    }
}

pub struct Gpu {
    // ***** GPU PARAMETERS ******
    // VRAM is a memory area used to store graphics such as backgrounds and sprites
    vram: [u8; VRAM_SIZE as usize],
    // tile set is a buffer computed by the GPU from VRAM at each write operation
    tile_set: [Tile; TILE_SET_SIZE as usize],
    // OAM is a memory area used to store sprites attributes
    // Sprites data are stored in VRAM memory $8000-8FFF
    oam: [u8; OAM_SIZE as usize],
    // object data is a buffer computed by the GPU from OAM at each write operation
    object_data: [ObjectData; NUMBER_OF_SPRITES],

    // ****** LCD DISPLAY PARAMETERS *******
    pub background_colors: BackgroundColors,
    pub viewport_x_offset: u8,
    pub viewport_y_offset: u8,
    pub lcd_display_enabled: bool,
    pub window_display_enabled: bool,
    pub background_display_enabled: bool,
    pub object_display_enabled: bool,
    pub line_equals_line_check_interrupt_enabled: bool,
    pub oam_interrupt_enabled: bool,
    pub vblank_interrupt_enabled: bool,
    pub hblank_interrupt_enabled: bool,
    pub line_check: u8,
    pub line_equals_line_check: bool,
    pub window_tile_map: TileMap,
    pub background_tile_map: TileMap,
    pub background_and_window_data_select: BackgroundAndWindowDataSelect,
    pub object_size: ObjectSize,
    pub obj_0_color_1: PixelColor,
    pub obj_0_color_2: PixelColor,
    pub obj_0_color_3: PixelColor,
    pub obj_1_color_1: PixelColor,
    pub obj_1_color_2: PixelColor,
    pub obj_1_color_3: PixelColor,
    pub window: Window,
    pub line: u8,
    pub mode: Mode,
    cycles: u16,

    // Output frame buffer
    pub frame_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            // GPU parameters
            vram: [0x00; VRAM_SIZE as usize],
            tile_set: [create_tile(); TILE_SET_SIZE as usize],
            oam: [0; OAM_SIZE as usize],
            object_data: [Default::default(); NUMBER_OF_SPRITES],

            // lCD parameters
            background_colors: BackgroundColors::new(),
            viewport_x_offset: 0,
            viewport_y_offset: 0,
            lcd_display_enabled: false,
            window_display_enabled: false,
            background_display_enabled: false,
            object_display_enabled: false,
            line_equals_line_check_interrupt_enabled: false,
            oam_interrupt_enabled: false,
            vblank_interrupt_enabled: false,
            hblank_interrupt_enabled: false,
            line_check: 0,
            line_equals_line_check: false,
            window_tile_map: TileMap::X9800,
            background_tile_map: TileMap::X9800,
            background_and_window_data_select: BackgroundAndWindowDataSelect::X8800,
            object_size: ObjectSize::OS8X8,
            obj_0_color_1: PixelColor::LIGHT_GRAY,
            obj_0_color_2: PixelColor::DARK_GRAY,
            obj_0_color_3: PixelColor::BLACK,
            obj_1_color_1: PixelColor::LIGHT_GRAY,
            obj_1_color_2: PixelColor::DARK_GRAY,
            obj_1_color_3: PixelColor::BLACK,
            window: Window { x: 0, y: 0 },
            line: 0,
            mode: Mode::HorizontalBlank,
            cycles: 0,

            // frame  buffer
            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],

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
                (true, true) => TilePixelValue::Three,
                (false, true) => TilePixelValue::Two,
                (true, false) => TilePixelValue::One,
                (false, false) => TilePixelValue::Zero,
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

    pub fn step(&mut self, cycles: u8) -> InterruptRequest {
        let mut request = InterruptRequest::None;
        if !self.lcd_display_enabled {
            return request;
        }
        self.cycles += cycles as u16;

        let mode = self.mode;
        match mode {
            Mode::HorizontalBlank => {
                if self.cycles >= 200 {
                    self.cycles = self.cycles % 200;
                    self.line += 1;

                    if self.line >= 144 {
                        self.mode = Mode::VerticalBlank;
                        request.add(InterruptRequest::VBlank);
                        if self.vblank_interrupt_enabled {
                            request.add(InterruptRequest::LCDStat)
                        }
                    } else {
                        self.mode = Mode::OAMAccess;
                        if self.oam_interrupt_enabled {
                            request.add(InterruptRequest::LCDStat)
                        }
                    }
                    self.set_equal_lines_check(&mut request);
                }
            }
            Mode::VerticalBlank => {
                if self.cycles >= 456 {
                    self.cycles = self.cycles % 456;
                    self.line += 1;
                    if self.line == 154 {
                        self.mode = Mode::OAMAccess;
                        self.line = 0;
                        if self.oam_interrupt_enabled {
                            request.add(InterruptRequest::LCDStat)
                        }
                    }
                    self.set_equal_lines_check(&mut request);
                }
            }
            Mode::OAMAccess => {
                if self.cycles >= 80 {
                    self.cycles = self.cycles % 80;
                    self.mode = Mode::VRAMAccess;
                }
            }
            Mode::VRAMAccess => {
                if self.cycles >= 172 {
                    self.cycles = self.cycles % 172;
                    if self.hblank_interrupt_enabled {
                        request.add(InterruptRequest::LCDStat)
                    }
                    self.mode = Mode::HorizontalBlank;
                    self.render_scan_line()
                }
            }
        }
        request
    }

    fn set_equal_lines_check(&mut self, request: &mut InterruptRequest) {
        let line_equals_line_check = self.line == self.line_check;
        if line_equals_line_check && self.line_equals_line_check_interrupt_enabled {
            request.add(InterruptRequest::LCDStat);
        }
        self.line_equals_line_check = line_equals_line_check;
    }

    fn render_scan_line(&mut self) {
        let mut scan_line: [TilePixelValue; SCREEN_WIDTH] = [Default::default(); SCREEN_WIDTH];
        if self.background_display_enabled {
            // The x index of the current tile
            let mut tile_x_index = self.viewport_x_offset / 8;
            // The current scan line's y-offset in the entire background space is a combination
            // of both the line inside the view port we're currently on and the amount of the view port is scrolled
            let tile_y_index = self.line.wrapping_add(self.viewport_y_offset);
            // The current tile we're on is equal to the total y offset broken up into 8 pixel chunks
            // and multipled by the width of the entire background (i.e. 32 tiles)
            let tile_offset = (tile_y_index as u16 / 8) * 32u16;

            // Where is our tile map defined?
            let background_tile_map = if self.background_tile_map == TileMap::X9800 {
                0x9800
            } else {
                0x9C00
            };
            // Munge this so that the beginning of VRAM is index 0
            let tile_map_begin = background_tile_map - VRAM_BEGIN;
            // Where we are in the tile map is the beginning of the tile map
            // plus the current tile's offset
            let tile_map_offset = (tile_map_begin + tile_offset) as usize;

            // When line and scrollY are zero we just start at the top of the tile
            // If they're non-zero we must index into the tile cycling through 0 - 7
            let row_y_offset = tile_y_index % 8;
            let mut pixel_x_index = self.viewport_x_offset % 8;

            if self.background_and_window_data_select == BackgroundAndWindowDataSelect::X8800 {
                panic!("TODO: support 0x8800 background and window data select");
            }

            let mut frame_buffer_offset = self.line as usize * SCREEN_WIDTH * 4;
            // Start at the beginning of the line and go pixel by pixel
            for line_x in 0..SCREEN_WIDTH {
                // Grab the tile index specified in the tile map
                let tile_index = self.vram[tile_map_offset + tile_x_index as usize];

                let tile_value = self.tile_set[tile_index as usize][row_y_offset as usize]
                    [pixel_x_index as usize];
                let color = self.tile_value_to_background_color(&tile_value);

                self.frame_buffer[frame_buffer_offset] = color as u8;
                self.frame_buffer[frame_buffer_offset + 1] = color as u8;
                self.frame_buffer[frame_buffer_offset + 2] = color as u8;
                self.frame_buffer[frame_buffer_offset + 3] = 255;
                frame_buffer_offset += 4;
                scan_line[line_x] = tile_value;
                // Loop through the 8 pixels within the tile
                pixel_x_index = (pixel_x_index + 1) % 8;

                // Check if we've fully looped through the tile
                if pixel_x_index == 0 {
                    // Now increase the tile x_offset by 1
                    tile_x_index = tile_x_index + 1;
                }
                if self.background_and_window_data_select == BackgroundAndWindowDataSelect::X8800 {
                    panic!("TODO: support 0x8800 background and window data select");
                }
            }
        }

        if self.object_display_enabled {
            let object_height = if self.object_size == ObjectSize::OS8X16 {
                16
            } else {
                8
            };
            for object in self.object_data.iter() {
                let line = self.line as i16;
                if object.y <= line && object.y + object_height > line {
                    let pixel_y_offset = line - object.y;
                    let tile_index = if object_height == 16 && (!object.yflip && pixel_y_offset > 7)
                        || (object.yflip && pixel_y_offset <= 7)
                    {
                        object.tile + 1
                    } else {
                        object.tile
                    };

                    let tile = self.tile_set[tile_index as usize];
                    let tile_row = if object.yflip {
                        tile[(7 - (pixel_y_offset % 8)) as usize]
                    } else {
                        tile[(pixel_y_offset % 8) as usize]
                    };

                    let canvas_y_offset = line as i32 * SCREEN_WIDTH as i32;
                    let mut canvas_offset = ((canvas_y_offset + object.x as i32) * 4) as usize;
                    for x in 0..8i16 {
                        let pixel_x_offset = if object.xflip { (7 - x) } else { x } as usize;
                        let x_offset = object.x + x;
                        let pixel = tile_row[pixel_x_offset];
                        if x_offset >= 0
                            && x_offset < SCREEN_WIDTH as i16
                            && pixel != TilePixelValue::Zero
                            && (object.priority
                                || scan_line[x_offset as usize] == TilePixelValue::Zero)
                        {
                            let color = self.tile_value_to_background_color(&pixel);

                            self.frame_buffer[canvas_offset + 0] = color as u8;
                            self.frame_buffer[canvas_offset + 1] = color as u8;
                            self.frame_buffer[canvas_offset + 2] = color as u8;
                            self.frame_buffer[canvas_offset + 3] = 255;
                        }
                        canvas_offset += 4;
                    }
                }
            }
        }

        if self.window_display_enabled {}
    }

    fn tile_value_to_background_color(&self, tile_value: &TilePixelValue) -> PixelColor {
        match tile_value {
            TilePixelValue::Zero => self.background_colors.0,
            TilePixelValue::One => self.background_colors.1,
            TilePixelValue::Two => self.background_colors.2,
            TilePixelValue::Three => self.background_colors.3,
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

        assert_eq!(gpu.tile_set[0][0][0], TilePixelValue::Three);
        assert_eq!(gpu.tile_set[0][0][5], TilePixelValue::One);
        assert_eq!(gpu.tile_set[0][0][2], TilePixelValue::Two);

        gpu.write_vram(0x00F0, 0xCC);
        gpu.write_vram(0x00F1, 0xAA);

        assert_eq!(gpu.tile_set[15][0][0], TilePixelValue::Three);
        assert_eq!(gpu.tile_set[15][0][5], TilePixelValue::One);
        assert_eq!(gpu.tile_set[15][0][2], TilePixelValue::Two);
    }

    #[test]
    fn test_create_tile() {
        let mut new_tile = create_tile();
        assert_eq!(new_tile[1][1], TilePixelValue::Zero);

        new_tile[1][2] = TilePixelValue::Two;
        assert_eq!(new_tile[1][2], TilePixelValue::Two);
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
