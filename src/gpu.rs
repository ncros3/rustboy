use crate::bus::{VRAM_BEGIN, VRAM_SIZE, OAM_SIZE};

const TILE_LENGHT: u8 = 8;
const TILE_SET_SIZE: u16 = 384;

const NUMBER_OF_SPRITES: usize = 40;
const SPRITE_LENGTH_IN_BYTE: usize = 4;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

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
pub enum DataSet {
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
pub enum GpuInterruptRequest {
    None,
    VBlank,
    LCDStat,
    Both,
}

impl GpuInterruptRequest {
    fn add(&mut self, other: GpuInterruptRequest) {
        match self {
            GpuInterruptRequest::None => *self = other,
            GpuInterruptRequest::VBlank if other == GpuInterruptRequest::LCDStat => {
                *self = GpuInterruptRequest::Both
            }
            GpuInterruptRequest::LCDStat if other == GpuInterruptRequest::VBlank => {
                *self = GpuInterruptRequest::Both
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
    // 0xFF40: LCD control register
    pub lcd_display_enabled: bool,
    pub window_tile_map: TileMap,
    pub window_display_enabled: bool,
    pub background_and_window_data_select: DataSet,
    pub background_tile_map: TileMap,
    pub object_size: ObjectSize,
    pub object_display_enabled: bool,

    pub background_display_enabled: bool,

    // 0xFF41: LCD status register 
    pub line_equals_line_check_interrupt_enabled: bool,
    pub oam_interrupt_enabled: bool,
    pub vblank_interrupt_enabled: bool,
    pub hblank_interrupt_enabled: bool,
    pub line_equals_line_check: bool,
    pub mode: Mode,

    // 0xFF42 - 0xFF43: SCY viewport Y offset
    pub viewport_y_offset: u8,
    pub viewport_x_offset: u8,

    // 0xFF44: LY 
    pub current_line: u8,

    // 0xFF45: LY compare
    pub compare_line: u8,

    // 0xFF47: Background palette
    pub background_colors: BackgroundColors,

    // 0xFF48: Objects palette 0
    pub obj_0_color_1: PixelColor,
    pub obj_0_color_2: PixelColor,
    pub obj_0_color_3: PixelColor,

    // 0xFF49: Objects palette 1
    pub obj_1_color_1: PixelColor,
    pub obj_1_color_2: PixelColor,
    pub obj_1_color_3: PixelColor,

    // 0xFF4A - 0xFF4B: window position
    pub window: Window,

    // ****** GPU GENERAL PARAMETERS *******
    cycles: u16,

    // ****** OUTPUT FRAME BUFFER *******
    pub frame_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
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
            compare_line: 0,
            line_equals_line_check: false,
            window_tile_map: TileMap::X9800,
            background_tile_map: TileMap::X9800,
            background_and_window_data_select: DataSet::X8800,
            object_size: ObjectSize::OS8X8,
            obj_0_color_1: PixelColor::LIGHT_GRAY,
            obj_0_color_2: PixelColor::DARK_GRAY,
            obj_0_color_3: PixelColor::BLACK,
            obj_1_color_1: PixelColor::LIGHT_GRAY,
            obj_1_color_2: PixelColor::DARK_GRAY,
            obj_1_color_3: PixelColor::BLACK,
            window: Window { x: 0, y: 0 },
            current_line: 0,
            mode: Mode::HorizontalBlank,
            cycles: 0,

            // frame  buffer
            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, data: u8) {
        self.vram[address as usize] = data;
    }

    pub fn write_oam(&mut self, index: usize, data: u8) {
        // save data in OAM memory
        self.oam[index] = data;
    }

    pub fn read_oam(&self, address: usize) -> u8 {
        self.oam[address]
    }

    pub fn run(&mut self, cycles: u8) -> GpuInterruptRequest {
        let mut request = GpuInterruptRequest::None;
        if !self.lcd_display_enabled {
            return request;
        }
        self.cycles += cycles as u16;

        let mode = self.mode;
        match mode {
            Mode::HorizontalBlank => {
                if self.cycles >= 200 {
                    self.cycles = self.cycles % 200;
                    self.current_line += 1;

                    if self.current_line >= 144 {
                        self.mode = Mode::VerticalBlank;
                        request.add(GpuInterruptRequest::VBlank);
                        if self.vblank_interrupt_enabled {
                            request.add(GpuInterruptRequest::LCDStat)
                        }
                    } else {
                        self.mode = Mode::OAMAccess;
                        if self.oam_interrupt_enabled {
                            request.add(GpuInterruptRequest::LCDStat)
                        }
                    }
                }
            }
            Mode::VerticalBlank => {
                if self.cycles >= 456 {
                    self.cycles = self.cycles % 456;
                    self.current_line += 1;
                    if self.current_line == 154 {
                        self.mode = Mode::OAMAccess;
                        self.current_line = 0;
                        if self.oam_interrupt_enabled {
                            request.add(GpuInterruptRequest::LCDStat)
                        }
                    }
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
                        request.add(GpuInterruptRequest::LCDStat)
                    }
                    self.mode = Mode::HorizontalBlank;
                    self.draw_line()
                }
            }
        }
        request
    }


    fn draw_line(&mut self) {
        let mut scan_line: [TilePixelValue; SCREEN_WIDTH] = [Default::default(); SCREEN_WIDTH];

        // display background from VRAM memory
        if self.background_display_enabled {
            
        }

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
    use minifb::{Key, Window, WindowOptions};

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

    #[test]
    fn test_draw_frame_buffer(){
        const SCALE_FACTOR: usize = 3;
        const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];
        const NUMBER_OF_PIXELS: usize = 23040;

        let mut gpu = Gpu::new();
        let mut cycles : u32 = 0;

        let mut window = Window::new(
            "Rustboy",
            WINDOW_DIMENSIONS[0],
            WINDOW_DIMENSIONS[1],
            WindowOptions::default(),
        )
        .unwrap();



        while window.is_open() && !window.is_key_down(Key::Escape) {
            // temporary buffer to print on the screen
            let mut buffer = [0; NUMBER_OF_PIXELS];
            // update cycles
            cycles += 1;

            // load data in gpu tile set
            for i in 0..NUMBER_OF_PIXELS/2 {
                gpu.frame_buffer[i] = 255;
            }

            // run the gpu for an entire frame
            gpu.run(1);

            // copy this frame from gpu frame buffer
            for i in 0..NUMBER_OF_PIXELS/2 {
                buffer[i] = (gpu.frame_buffer[i] as u32) << 24
                            | (gpu.frame_buffer[i] as u32) << 16
                            | (gpu.frame_buffer[i] as u32) << 8
                            | 255 << 0;
            }

            // display the frame rendered by the gpu
            window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
        }
    
    }
}
