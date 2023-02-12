use crate::bus::{VRAM_BEGIN, VRAM_SIZE, OAM_SIZE};

const OBJECT_X_OFFSET: i16 = -8;
const OBJECT_Y_OFFSET: i16 = -16;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const TILE_ROW_SIZE_IN_PIXEL: u8 = 8;
const TILE_SIZE_IN_BYTES: u16 = 16;
const TILE_MAP_SIZE: u8 = 32;

const BYTES_PER_TILE_ROM: u8 = 2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PixelColor {
    WHITE = 255,
    LIGHT_GRAY = 192,
    DARK_GRAY = 96,
    BLACK = 0,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Palette(PixelColor, PixelColor, PixelColor, PixelColor);

impl Palette {
    fn new() -> Palette {
        Palette(
            PixelColor::WHITE,
            PixelColor::LIGHT_GRAY,
            PixelColor::DARK_GRAY,
            PixelColor::BLACK,
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ObjectData {
    x: i16,
    y: i16,
    tile: u8,
    palette: Palette,
    xflip: bool,
    yflip: bool,
    priority: bool,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum TileMapArea {
    X9800 = 0x1800,
    X9C00 = 0x1C00,
}

#[derive(Eq, PartialEq)]
pub enum ObjectSize {
    OS8X8,
    OS8X16,
}

#[derive(Copy, Clone)]
pub enum Mode {
    HorizontalBlank,
    VerticalBlank,
    OAMAccess,
    VRAMAccess,
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
    // OAM is a memory area used to store sprites attributes
    // Sprites data are stored in VRAM memory $8000-8FFF
    oam: [u8; OAM_SIZE as usize],

    // ****** LCD DISPLAY PARAMETERS *******
    // 0xFF40: LCD control register
    pub lcd_display_enabled: bool,
    pub window_tile_map: TileMapArea,
    pub window_display_enabled: bool,
    pub background_tile_data_area: bool,
    pub background_tile_map_area: TileMapArea,
    pub object_size: ObjectSize,
    pub object_display_enabled: bool,
    pub background_display_enabled: bool,

    // 0xFF41: LCD status register 
    pub line_compare_it_enable: bool,
    pub oam_interrupt_enabled: bool,
    pub vblank_interrupt_enabled: bool,
    pub hblank_interrupt_enabled: bool,
    pub line_compare_state: bool,
    pub mode: Mode,

    // 0xFF42 - 0xFF43: SCY viewport Y offset
    pub viewport_y_offset: u8,
    pub viewport_x_offset: u8,

    // 0xFF44: LY 
    pub current_line: u8,

    // 0xFF45: LY compare
    pub compare_line: u8,

    // 0xFF47: Background palette
    pub background_palette: Palette,

    // 0xFF48: Objects palette 0
    pub object_palette_0: Palette,

    // 0xFF49: Objects palette 1
    pub object_palette_1: Palette,

    // 0xFF4A - 0xFF4B: window position
    window_x_offset: u8,
    window_y_offset: u8,

    // ****** GPU GENERAL PARAMETERS *******
    cycles: u16,

    // ****** OUTPUT FRAME BUFFER *******
    pub frame_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            vram: [0x00; VRAM_SIZE as usize],
            oam: [0; OAM_SIZE as usize],

            lcd_display_enabled: false,
            window_tile_map: TileMapArea::X9800,
            window_display_enabled: false,
            background_tile_data_area: false,
            background_tile_map_area: TileMapArea::X9800,
            object_size: ObjectSize::OS8X8,
            object_display_enabled: false,
            background_display_enabled: false,

            line_compare_it_enable: false,
            oam_interrupt_enabled: false,
            vblank_interrupt_enabled: false,
            hblank_interrupt_enabled: false,
            line_compare_state: false,
            mode: Mode::HorizontalBlank,

            viewport_y_offset: 0,
            viewport_x_offset: 0,

            current_line: 0,
            compare_line: 0,

            background_palette: Palette::new(),
            object_palette_0: Palette::new(),
            object_palette_1: Palette::new(),

            window_x_offset: 0,
            window_y_offset: 0,

            cycles: 0,

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
        if self.background_display_enabled {
            let pixel_y_index = self.current_line - 1;

            for pixel_x_index in 0..SCREEN_WIDTH {
                // compute the tile index in tile map
                let tile_map_y_index = (pixel_y_index / TILE_ROW_SIZE_IN_PIXEL) as u16;
                let tile_map_x_index = (pixel_x_index / (TILE_ROW_SIZE_IN_PIXEL as usize)) as u16;
                let tile_map_index = tile_map_y_index * (TILE_MAP_SIZE as u16) + tile_map_x_index;

                // get the tile memory address from the tile map
                let tile_mem_index = self.read_vram((self.background_tile_map_area as u16) + tile_map_index);
                // convert a 8 bits tile index into a 16 bits tile memory addr
                let tile_mem_addr = (tile_mem_index as u16) * TILE_SIZE_IN_BYTES;

                // get the row offset in the tile
                let tile_row_offset = pixel_y_index % TILE_ROW_SIZE_IN_PIXEL * BYTES_PER_TILE_ROM;

                // get tile row data from vram
                let (data_1, data_0) = self.get_tile_data(tile_mem_addr, tile_row_offset as u16);

                // get pixel bits from data
                let bit_0 = data_0 >> (7 - (pixel_x_index % (TILE_ROW_SIZE_IN_PIXEL as usize))) & 0x01;
                let bit_1 = data_1 >> (7 - (pixel_x_index % (TILE_ROW_SIZE_IN_PIXEL as usize))) & 0x01;

                // find pixel color
                let pixel_value = (bit_1 << 1) | bit_0;
                let pixel_color = self.get_bg_pixel_color_from_palette(pixel_value);

                // fill frame buffer
                self.frame_buffer[(pixel_y_index as usize) * SCREEN_WIDTH + (pixel_x_index as usize)] = pixel_color;
            }
        }
    }

    fn get_tile_data(&self, tile_mem_addr: u16, tile_row_offset: u16) -> (u8, u8) {

        if self.background_tile_data_area {
            // $8000 method addressing
            let data_0 = self.read_vram(tile_mem_addr + tile_row_offset);
            let data_1 = self.read_vram(tile_mem_addr + tile_row_offset + 1);

            return (data_1, data_0);
        } else {
            // $8800 method adressing
            if (tile_mem_addr + tile_row_offset) < 0x0800 {
                let data_0 = self.read_vram(0x1000 + tile_mem_addr + tile_row_offset);
                let data_1 = self.read_vram(0x1000 + tile_mem_addr + tile_row_offset + 1);

                return (data_1, data_0);
            } else {
                let data_0 = self.read_vram(tile_mem_addr + tile_row_offset);
                let data_1 = self.read_vram(tile_mem_addr + tile_row_offset + 1);

                return (data_1, data_0);
            }
        }
    }

    fn get_bg_pixel_color_from_palette(&self, pixel_value: u8) -> u8 {
        match pixel_value {
            0 => self.background_palette.0 as u8,
            1 => self.background_palette.1 as u8,
            2 => self.background_palette.2 as u8,
            3 => self.background_palette.3 as u8,
            _ => self.background_palette.0 as u8,
        }
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;
    use minifb::{Key, Window, WindowOptions};

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
    fn test_draw_line() {
        let mut gpu = Gpu::new();

        // init GPU
        gpu.background_display_enabled = true;
        gpu.background_tile_data_area = true;
        gpu.background_tile_map_area = TileMapArea::X9800;
        gpu.current_line = 9; // first line of the second tile row

        // init VRAM
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x0200, 0x80);
        gpu.write_vram(0x0201, 0x80);
        gpu.write_vram(0x0210, 0x80);
        gpu.write_vram(0x0211, 0x80);

        // set tile map
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x1820, 0x20);
        gpu.write_vram(0x1821, 0x21);

        // draw the line in the frame buffer
        gpu.draw_line();

        // check frame buffer
        // line 9 * 160 = 1440 / 0x05A0
        assert_eq!(gpu.frame_buffer[0x0500], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0508], PixelColor::BLACK as u8);
    }

    #[test]
    fn test_tile_data_area_2() {
        let mut gpu = Gpu::new();

        // init GPU
        gpu.background_display_enabled = true;
        gpu.background_tile_data_area = false;
        gpu.background_tile_map_area = TileMapArea::X9800;

        // init VRAM
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x1200, 0x80);
        gpu.write_vram(0x1201, 0x80);
        gpu.write_vram(0x1210, 0x80);
        gpu.write_vram(0x1211, 0x80);

        // here we're looking for tile at index 128 and 129
        gpu.write_vram(0x0800, 0x80);
        gpu.write_vram(0x0801, 0x80);
        gpu.write_vram(0x0810, 0x80);
        gpu.write_vram(0x0811, 0x80);


        // set tile map
        // here we're looking for tile map at index 32 and 33 / line 9
        // which redirects to tile data index 32 and 33
        gpu.write_vram(0x1820, 0x20);
        gpu.write_vram(0x1821, 0x21);
        // here we're looking for tile map at index 512 and 513 / line 129
        // which redirects to tile data index 128 and 129
        gpu.write_vram(0x1A00, 0x80);
        gpu.write_vram(0x1A01, 0x81);

        // draw the line in the frame buffer
        gpu.current_line = 9; // first line of the second tile row -> line 9
        gpu.draw_line();

        gpu.current_line = 129; // first line of the 16th tile row -> line 129
        gpu.draw_line();

        // check frame buffer
        // line 8 * 160 = 1440 / 0x0500
        assert_eq!(gpu.frame_buffer[0x0500], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0508], PixelColor::BLACK as u8);
        // line 128 * 160 = 1440 / 0x0500
        assert_eq!(gpu.frame_buffer[0x5000], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x5008], PixelColor::BLACK as u8);
    }

    // #[test]
    // fn test_draw_frame_buffer(){
    //     const SCALE_FACTOR: usize = 3;
    //     const WINDOW_DIMENSIONS: [usize; 2] = [(SCREEN_WIDTH * SCALE_FACTOR), (SCREEN_HEIGHT * SCALE_FACTOR)];
    //     const NUMBER_OF_PIXELS: usize = 23040;

    //     let mut gpu = Gpu::new();
    //     let mut cycles : u32 = 0;

    //     let mut window = Window::new(
    //         "Rustboy",
    //         WINDOW_DIMENSIONS[0],
    //         WINDOW_DIMENSIONS[1],
    //         WindowOptions::default(),
    //     )
    //     .unwrap();

    //     while window.is_open() && !window.is_key_down(Key::Escape) {
    //         // temporary buffer to print on the screen
    //         let mut buffer = [0; NUMBER_OF_PIXELS];
    //         // update cycles
    //         cycles += 1;

    //         // load data in gpu tile set
    //         for i in 0..NUMBER_OF_PIXELS/2 {
    //             gpu.frame_buffer[i] = 155;
    //         }

    //         // run the gpu for an entire frame
    //         gpu.run(1);

    //         // copy this frame from gpu frame buffer
    //         for i in 0..NUMBER_OF_PIXELS/2 {
    //             buffer[i] =  255 << 24
    //                         | (gpu.frame_buffer[i] as u32) << 16
    //                         | (gpu.frame_buffer[i] as u32) << 8
    //                         | (gpu.frame_buffer[i] as u32) << 0;
    //         }

    //         // display the frame rendered by the gpu
    //         window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    //     }
    
    // }
}
