use crate::soc::peripheral::{VRAM_SIZE, OAM_SIZE};
use crate::soc::peripheral::nvic::{Nvic, InterruptSources};

const HORIZONTAL_BLANK_CYCLES: u16 = 204;
const VERTICAL_BLANK_CYCLES: u16 = 4560;
const OAM_SCAN_CYCLES: u16 = 80;
const DRAW_PIXEL_CYCLES: u16 = 172;
const ONE_LINE_CYCLES: u16 = HORIZONTAL_BLANK_CYCLES + OAM_SCAN_CYCLES + DRAW_PIXEL_CYCLES;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const TILE_ROW_SIZE_IN_PIXEL: u8 = 8;
const TILE_SIZE_IN_BYTES: u16 = 16;
const TILE_MAP_SIZE: u8 = 32;

const BYTES_PER_TILE_ROM: u8 = 2;

const SPRITE_X_OFFSET: i16 = 8;
const SPRITE_Y_OFFSET: i16 = 16;

const NB_SPRITES_IN_OAM: u16 = 40;
const SPRITE_ATTRIBUTES_SIZE_IN_BYTES: u16 = 4;
const SPRITE_Y_POS_OFFSET: u16 = 0;
const SPRITE_X_POS_OFFSET: u16 = 1;
const SPRITE_TILE_INDEX_OFFSET: u16 = 2;
const SPRITE_ATTRIBUTES_OFFSET: u16 = 3;
const NB_SRITES_TO_DISPLAY_MAX: u16 = 10;
const PIXEL_TRANSPARENT: u8 = 0x00;

const WINDOW_X_OFFSET: u8 = 7;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PixelColor {
    WHITE = 255,
    LIGHT_GRAY = 192,
    DARK_GRAY = 96,
    BLACK = 0,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Palette{
    pub color_0: PixelColor, 
    pub color_1: PixelColor, 
    pub color_2: PixelColor, 
    pub color_3: PixelColor,
}

impl Palette {
    fn new() -> Palette {
        Palette {
            color_0: PixelColor::WHITE,
            color_1: PixelColor::LIGHT_GRAY,
            color_2: PixelColor::DARK_GRAY,
            color_3: PixelColor::BLACK,
        }
    }
}

macro_rules! set_palette {
    ($self:ident.$palette:ident.$palette_index:ident, $data: ident, $color_index: expr) => {{
        let value = ($data >> ($color_index * 2)) & 0x03;

        match value {
            0 => $self.$palette.$palette_index = PixelColor::WHITE,
            1 => $self.$palette.$palette_index = PixelColor::LIGHT_GRAY,
            2 => $self.$palette.$palette_index = PixelColor::DARK_GRAY,
            _ => $self.$palette.$palette_index = PixelColor::BLACK,
        }
    }};
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GpuMode {
    HorizontalBlank,
    VerticalBlank,
    OAMScan,
    DrawPixel,
}

pub struct Gpu {
    // ***** GPU PARAMETERS ******
    // VRAM is a memory area used to store graphics such as backgrounds and sprites
    pub vram: [u8; VRAM_SIZE as usize],
    // OAM is a memory area used to store sprites attributes
    // Sprites data are stored in VRAM memory $8000-8FFF
    oam: [u8; OAM_SIZE as usize],

    // ****** LCD DISPLAY PARAMETERS *******
    // 0xFF40: LCD control register
    pub lcd_display_enabled: bool,
    pub window_tile_map_area: TileMapArea,
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
    pub mode: GpuMode,

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

    // ****** GPU INTERNAL PARAMETERS *******
    cycles: u16,
    new_mode_flag: bool,
    vblank_line: u16,

    // ****** OUTPUT FRAME BUFFER *******
    pub frame_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            vram: [0xFF; VRAM_SIZE as usize],
            oam: [0xFF; OAM_SIZE as usize],

            lcd_display_enabled: false,
            window_tile_map_area: TileMapArea::X9800,
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
            mode: GpuMode::OAMScan,

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
            new_mode_flag: true,
            vblank_line: 0,

            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, data: u8) { 
        self.vram[address as usize] = data;
    }

    pub fn read_oam(&self, address: usize) -> u8 {
        self.oam[address]
    }

    pub fn write_oam(&mut self, address: usize, data: u8) {         
        self.oam[address] = data;
    }

    pub fn run(&mut self, cycles: u8, nvic: &mut Nvic) {
        if self.lcd_display_enabled {
            // update GPU cycles counter
            self.cycles += cycles as u16;

            match self.mode {
                GpuMode::HorizontalBlank => {
                    // handle interrupts generation
                    if self.new_mode_flag && self.hblank_interrupt_enabled{
                        self.new_mode_flag = false;
                        nvic.set_interrupt(InterruptSources::STAT);
                    }

                    // we reached the end of the mode
                    if self.cycles >= HORIZONTAL_BLANK_CYCLES {
                        self.cycles = self.cycles % HORIZONTAL_BLANK_CYCLES;
                        // we detected the end of a line
                        if self.current_line < (SCREEN_HEIGHT - 1) as u8 {
                            self.current_line += 1;
                            // run the compare line circuitry
                            self.compare_line(nvic);
                            // reset new mode flag
                            self.new_mode_flag = true;
                            // go to next gpu mode
                            self.mode = GpuMode::OAMScan;
                        } else {
                            // reset new mode flag
                            self.new_mode_flag = true;
                            // go to next gpu mode
                            self.mode = GpuMode::VerticalBlank;
                        }
                    }
                }
                GpuMode::VerticalBlank => {
                    // handle interrupts generation
                    if self.new_mode_flag {
                        self.new_mode_flag = false;
                        nvic.set_interrupt(InterruptSources::VBLANK);

                        if self.vblank_interrupt_enabled {
                            nvic.set_interrupt(InterruptSources::STAT);
                        }
                    }

                    // if we reached a new line in vblank mode, run compare line circuitry
                    if (self.cycles / ((self.vblank_line + 1) * ONE_LINE_CYCLES)) != 0 {
                        self.vblank_line += 1;
                        self.current_line += 1;

                        self.compare_line(nvic);
                    }

                    // we reached the end of the mode
                    if self.cycles >= VERTICAL_BLANK_CYCLES {
                        self.cycles = self.cycles % VERTICAL_BLANK_CYCLES;
                        // reset the line counter to draw a new frame
                        self.current_line = 0;
                        // reset the vblank line counter
                        self.vblank_line = 0;
                        // reset new mode flag
                        self.new_mode_flag = true;
                        // go to next gpu mode
                        self.mode = GpuMode::OAMScan;
                    }
                }
                GpuMode::OAMScan => {
                    // handle interrupts generation
                    if self.new_mode_flag && self.oam_interrupt_enabled{
                        self.new_mode_flag = false;
                        nvic.set_interrupt(InterruptSources::STAT);
                    }

                    // we reached the end of the mode
                    if self.cycles >= OAM_SCAN_CYCLES {
                        self.cycles = self.cycles % OAM_SCAN_CYCLES;
                        // reset new mode flag
                        self.new_mode_flag = true;
                        // go to next gpu mode
                        self.mode = GpuMode::DrawPixel;
                    }
                }
                GpuMode::DrawPixel => {
                    // we reached the end of the mode
                    if self.cycles >= DRAW_PIXEL_CYCLES {
                        self.cycles = self.cycles % DRAW_PIXEL_CYCLES;
                        // draw the line at the end of the draw pixel mode
                        self.draw_line();
                        // go to next gpu mode
                        self.mode = GpuMode::HorizontalBlank;
                    }
                }
            }
        }
    }


    fn draw_line(&mut self) {
        let mut bg_line = [0x00; SCREEN_WIDTH as usize];

        if self.background_display_enabled  {
            let pixel_y_index: u8 = self.current_line;

            for pixel_x_index in 0..SCREEN_WIDTH {
                // check if we display the background or the window
                let (tile_map_area, y_offset, x_offset) = 
                    if self.window_display_enabled 
                    && self.window_y_offset < self.current_line
                    && self.window_x_offset < pixel_x_index as u8 {
                        // window display mode
                        (self.window_tile_map_area,
                        self.window_y_offset,
                        self.window_x_offset.wrapping_sub(WINDOW_X_OFFSET))
                    } else {
                        // background display mode
                        (self.background_tile_map_area,
                        self.viewport_y_offset,
                        self.viewport_x_offset)
                    };

                // compute the tile index in tile map
                let tile_map_y_index = (pixel_y_index.wrapping_add(y_offset) / TILE_ROW_SIZE_IN_PIXEL) as u16;
                let tile_map_x_index = (((pixel_x_index as u8).wrapping_add(x_offset) as usize) / (TILE_ROW_SIZE_IN_PIXEL as usize)) as u16;
                let tile_map_index = tile_map_y_index * (TILE_MAP_SIZE as u16) + tile_map_x_index;

                // get the tile memory address from the tile map
                let tile_mem_index = self.read_vram((tile_map_area as u16) + tile_map_index);

                // convert a 8 bits tile index into a 16 bits tile memory addr
                let tile_mem_addr = (tile_mem_index as u16) * TILE_SIZE_IN_BYTES;

                // get the row offset in the tile
                let tile_row_offset = pixel_y_index.wrapping_add(y_offset) % TILE_ROW_SIZE_IN_PIXEL * BYTES_PER_TILE_ROM;

                // get tile row data from vram
                let (data_1, data_0) = self.get_bg_tile_data(tile_mem_addr, tile_row_offset as u16);

                // get pixel bits from data
                let bit_0 = data_0 >> (7 - (((pixel_x_index as u8).wrapping_add(x_offset) as usize) % (TILE_ROW_SIZE_IN_PIXEL as usize))) & 0x01;
                let bit_1 = data_1 >> (7 - (((pixel_x_index as u8).wrapping_add(x_offset) as usize) % (TILE_ROW_SIZE_IN_PIXEL as usize))) & 0x01;

                // find pixel color
                let pixel_value = (bit_1 << 1) | bit_0;
                let pixel_color = self.get_bg_pixel_color_from_palette(pixel_value);

                // fill frame buffer
                self.frame_buffer[(pixel_y_index as usize) * SCREEN_WIDTH + pixel_x_index] = pixel_color;
                // save the line for sprite rendering
                bg_line[pixel_x_index] = pixel_value;
            }
        }

        if self.object_display_enabled {
            // sprites array wich will contain sprites address to display
            let mut sprites: Vec<u16> = Vec::new();
            // find 10 sprites to display on the current line
            let mut nb_sprites_to_display = 0;
            for sprites_idx in 0..NB_SPRITES_IN_OAM {
                if nb_sprites_to_display < NB_SRITES_TO_DISPLAY_MAX {
                    let sprite_addr = sprites_idx * SPRITE_ATTRIBUTES_SIZE_IN_BYTES;
                    // get the srite first line
                    let sprite_y_pos_start = self.read_oam((sprite_addr + SPRITE_Y_POS_OFFSET) as usize) as u16 as i16 - SPRITE_Y_OFFSET;
                    // get the sprite last line
                    let sprite_y_pos_end = match self.object_size {
                        ObjectSize::OS8X8 => sprite_y_pos_start + TILE_ROW_SIZE_IN_PIXEL as i16 - 1,
                        ObjectSize::OS8X16 => sprite_y_pos_start + TILE_ROW_SIZE_IN_PIXEL as i16 * 2 - 1,
                    };
                    // check if the current line hits the sprite
                    if (self.current_line as i16 >= sprite_y_pos_start) && (self.current_line as i16 <= sprite_y_pos_end) {
                        // add the sprite to the list
                        sprites.push(sprite_addr);
                        // increase sprites counter
                        nb_sprites_to_display += 1;
                    }
                } else {
                    // we reached the maximum sprites number to display
                    // go out of the loop
                    break
                }
            }
            // sort objects to draw :
            // from lower priority in first positions
            // to higher priority in last positions
            let mut sprites_sorted: Vec<u16> = Vec::new();
            for sprite_idx in 0..nb_sprites_to_display {
                sprites_sorted.push(sprites[(nb_sprites_to_display - 1 - sprite_idx) as usize]);
            }
            // draw sorted sprites
            // higher priority sprites are drawn in last positions
            // so it can override lower priority sprites values
            for sprite in sprites_sorted {
                let pixel_y_index: u8 = self.current_line;
                // get sprite's attributes
                let sprite_y_pos = self.read_oam((sprite + SPRITE_Y_POS_OFFSET) as usize) as u16 as i16  - SPRITE_Y_OFFSET;
                let sprite_x_pos = self.read_oam((sprite + SPRITE_X_POS_OFFSET) as usize) as i16;
                let sprite_tile_addr = match self.object_size {
                    ObjectSize::OS8X8 => {
                        self.read_oam((sprite + SPRITE_TILE_INDEX_OFFSET) as usize) as u16 * TILE_SIZE_IN_BYTES
                    },
                    ObjectSize::OS8X16 => {
                        // ignore bit 0 for tile index in 8x16 object size mode
                        (self.read_oam((sprite + SPRITE_TILE_INDEX_OFFSET) as usize) as u16 * TILE_SIZE_IN_BYTES) & 0xFFE0
                    },
                };
                let sprite_attr = self.read_oam((sprite + SPRITE_ATTRIBUTES_OFFSET) as usize);
                let sprite_bg_over = (sprite_attr & 0x80) != 0;
                let sprite_y_flip = (sprite_attr & 0x40) != 0;
                let sprite_x_flip = (sprite_attr & 0x20) != 0;
                let sprite_palette_idx = (sprite_attr & 0x10) != 0;
                let sprite_size_offset =  match self.object_size {
                    ObjectSize::OS8X8 => 1,
                    ObjectSize::OS8X16 => 2,
                };
                // get tile addr
                let sprite_row_offset = (pixel_y_index as i16 - sprite_y_pos) as u16;
                let tile_addr = if sprite_y_flip == false {
                    sprite_tile_addr + sprite_row_offset * BYTES_PER_TILE_ROM as u16
                } else {
                    let row = ((TILE_ROW_SIZE_IN_PIXEL * sprite_size_offset) as u16).wrapping_sub(1).wrapping_sub(sprite_row_offset);
                    sprite_tile_addr + row * BYTES_PER_TILE_ROM as u16
                };
                // get one row of sprite data
                let data_0 = self.read_vram(tile_addr);
                let data_1 = self.read_vram(tile_addr + 1);
                // draw each pixel of the sprite's row
                for pixel_x_offset in 0..TILE_ROW_SIZE_IN_PIXEL {
                    // get pixel bits from data
                    let (bit_1, bit_0) = if sprite_x_flip == false {
                        let bit_0 = (data_0 >> (7 - pixel_x_offset)) & 0x01;
                        let bit_1 = (data_1 >> (7 - pixel_x_offset)) & 0x01;

                        (bit_1, bit_0)
                    } else {
                        let bit_0 = (data_0 >> pixel_x_offset) & 0x01;
                        let bit_1 = (data_1 >> pixel_x_offset) & 0x01;

                        (bit_1, bit_0)
                    };
                    // deduce pixel value
                    let pixel_value = (bit_1 << 1) | bit_0;
                    // compute the x coordinate of the pixel in the frame buffer
                    let pixel_x_index = sprite_x_pos - SPRITE_X_OFFSET + pixel_x_offset as i16;
                    // don't draw the pixel if it's not in the viewport
                    if pixel_x_index >= 0 
                    && pixel_x_index < SCREEN_WIDTH as i16 
                    && pixel_value != PIXEL_TRANSPARENT {
                        // check if bg overlap sprites
                        if !sprite_bg_over || bg_line[pixel_x_index as usize] == PIXEL_TRANSPARENT {
                            // find sprite pixel color
                            let pixel_color = self.get_object_pixel_color_from_palette(pixel_value, sprite_palette_idx);
                            // fill frame buffer
                            self.frame_buffer[(pixel_y_index as usize) * SCREEN_WIDTH + (pixel_x_index as usize)] = pixel_color;
                        } else {
                            // find bg pixel color
                            let pixel_color = self.get_bg_pixel_color_from_palette(bg_line[pixel_x_index as usize]);
                            // fill frame buffer
                            self.frame_buffer[(pixel_y_index as usize) * SCREEN_WIDTH + (pixel_x_index as usize)] = pixel_color;
                        }
                    }
                }
            }
        }
    }

    fn get_bg_tile_data(&self, tile_mem_addr: u16, tile_row_offset: u16) -> (u8, u8) {

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

    pub fn get_bg_pixel_color_from_palette(&self, pixel_value: u8) -> u8 {
        match pixel_value {
            0 => self.background_palette.color_0 as u8,
            1 => self.background_palette.color_1 as u8,
            2 => self.background_palette.color_2 as u8,
            _ => self.background_palette.color_3 as u8,
        }
    }

    fn get_object_pixel_color_from_palette(&self, pixel_value: u8, sprite_palette_idx: bool) -> u8 {
        if sprite_palette_idx {
            match pixel_value {
                0 => self.object_palette_1.color_0 as u8,
                1 => self.object_palette_1.color_1 as u8,
                2 => self.object_palette_1.color_2 as u8,
                _ => self.object_palette_1.color_3 as u8,
            }
        } else {
            match pixel_value {
                0 => self.object_palette_0.color_0 as u8,
                1 => self.object_palette_0.color_1 as u8,
                2 => self.object_palette_0.color_2 as u8,
                _ => self.object_palette_0.color_3 as u8,
            }
        }
    }

    fn compare_line(&mut self, nvic: &mut Nvic) {
        if self.current_line == self.compare_line {
            self.line_compare_state = true;

            // managed interrupt
            if self.line_compare_it_enable {
                nvic.set_interrupt(InterruptSources::STAT);
            }
        } else {
            self.line_compare_state = false;
        }
    }

    pub fn control_from_byte(&mut self, data: u8) {
        // bit 7
        self.lcd_display_enabled = ((data >> 7) & 0x01) != 0;
        // bit 6
        if((data >> 6) & 0x01) != 0 {
            self.window_tile_map_area = TileMapArea::X9C00;            
        } else {
            self.window_tile_map_area = TileMapArea::X9800;
        }
        // bit 5
        self.window_display_enabled = ((data >> 5) & 0x01) != 0;
        // bit 4
        self.background_tile_data_area = ((data >> 4) & 0x01) != 0;
        // bit 3
        if((data >> 3) & 0x01) != 0 {
            self.background_tile_map_area = TileMapArea::X9C00;         
        } else {
            self.background_tile_map_area = TileMapArea::X9800;
        }
        // bit 2
        if((data >> 2) & 0x01) != 0 {
            self.object_size = ObjectSize::OS8X16;       
        } else {
            self.object_size = ObjectSize::OS8X8;
        }
        // bit 1
        self.object_display_enabled = ((data >> 1) & 0x01) != 0;
        // bit 0
        self.background_display_enabled = (data & 0x01) != 0;
    }

    pub fn control_to_byte(&self) -> u8 {
        let window_tile_map_area_bit: u8 = if self.window_tile_map_area == TileMapArea::X9C00 {
            1
        } else {
            0
        };

        let background_tile_map_area_bit: u8 = if self.background_tile_map_area == TileMapArea::X9C00 {
            1
        } else {
            0
        };

        let object_size_bit: u8 = if self.object_size == ObjectSize::OS8X16 {
            1
        } else {
            0
        };

        ((self.lcd_display_enabled as u8) << 7)
            | (window_tile_map_area_bit << 6)
            | ((self.window_display_enabled as u8) << 5)
            | ((self.background_tile_data_area as u8) << 4)
            | (background_tile_map_area_bit << 3)
            | (object_size_bit << 2)
            | ((self.object_display_enabled as u8) << 1)
            | (self.background_display_enabled as u8)
    }

    pub fn status_from_byte(&mut self, data: u8) {
        self.line_compare_it_enable = ((data >> 6) & 0x01) != 0;
        self.oam_interrupt_enabled = ((data >> 5) & 0x01) != 0;
        self.vblank_interrupt_enabled = ((data >> 4) & 0x01) != 0;
        self.hblank_interrupt_enabled = ((data >> 3) & 0x01) != 0;
    }

    pub fn status_to_byte(&self) -> u8 {
        let gpu_mode_bits = match self.mode {
            GpuMode::HorizontalBlank => 0,
            GpuMode::VerticalBlank => 1,
            GpuMode::OAMScan => 2,
            GpuMode::DrawPixel => 3,
        };

        0x80
            | ((self.line_compare_it_enable as u8) << 6)
            | ((self.oam_interrupt_enabled as u8) << 5)
            | ((self.vblank_interrupt_enabled as u8) << 4)
            | ((self.hblank_interrupt_enabled as u8) << 3)
            | ((self.line_compare_state as u8) << 2)
            | ((gpu_mode_bits as u8) & 0x11)
    }

    pub fn get_scy(&self) -> u8 {
        self.viewport_y_offset
    }

    pub fn get_scx(&self) -> u8 {
        self.viewport_x_offset
    }

    pub fn get_current_line(&self) -> u8 {
        self.current_line
    }

    pub fn get_compare_line(&self) -> u8 {
        self.compare_line
    }

    pub fn set_scy(&mut self, data: u8) {
        self.viewport_y_offset = data;
    }

    pub fn set_scx(&mut self, data: u8) {
        self.viewport_x_offset = data;
    }

    pub fn set_compare_line(&mut self, data: u8) {
        self.compare_line = data;
    }

    pub fn set_window_y(&mut self, data: u8) {
        self.window_y_offset = data;
    }

    pub fn set_window_x(&mut self, data: u8) {
        self.window_x_offset = data;
    }

    pub fn get_window_y(&self) -> u8 {
        self.window_y_offset
    }

    pub fn get_window_x(&self) -> u8 {
        self.window_x_offset
    }

    pub fn set_background_palette(&mut self, data: u8) {
        set_palette!(self.background_palette.color_0, data, 0);
        set_palette!(self.background_palette.color_1, data, 1);
        set_palette!(self.background_palette.color_2, data, 2);
        set_palette!(self.background_palette.color_3, data, 3);
    }

    pub fn set_object_palette_0(&mut self, data: u8) {
        set_palette!(self.object_palette_0.color_0, data, 0);
        set_palette!(self.object_palette_0.color_1, data, 1);
        set_palette!(self.object_palette_0.color_2, data, 2);
        set_palette!(self.object_palette_0.color_3, data, 3);
    }

    pub fn set_object_palette_1(&mut self, data: u8) {
        set_palette!(self.object_palette_1.color_0, data, 0);
        set_palette!(self.object_palette_1.color_1, data, 1);
        set_palette!(self.object_palette_1.color_2, data, 2);
        set_palette!(self.object_palette_1.color_3, data, 3);
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;

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
        gpu.current_line = 8; // first line of the second tile row

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
        // line 8 * 160 = 1280 / 0x0500
        assert_eq!(gpu.frame_buffer[0x0500], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0508], PixelColor::BLACK as u8);
    }

    #[test]
    fn test_tile_data_area() {
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
        gpu.current_line = 8; // first line of the second tile row -> line 9
        gpu.draw_line();

        gpu.current_line = 128; // first line of the 16th tile row -> line 129
        gpu.draw_line();

        // check frame buffer
        // line 8 * 160 = 1280 / 0x0500
        assert_eq!(gpu.frame_buffer[0x0500], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0508], PixelColor::BLACK as u8);
        // line 128 * 160 = 20480 / 0x5000
        assert_eq!(gpu.frame_buffer[0x5000], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x5008], PixelColor::BLACK as u8);
    }

    #[test]
    fn test_tile_map_area() {
        let mut gpu = Gpu::new();

        // init GPU
        gpu.background_display_enabled = true;
        gpu.background_tile_data_area = true;
        gpu.background_tile_map_area = TileMapArea::X9800;
        gpu.current_line = 8; // first line of the second tile row

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
        // line 8 * 160 = 1280 / 0x0500
        assert_eq!(gpu.frame_buffer[0x0500], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0508], PixelColor::BLACK as u8);
    }

    #[test]
    fn test_scrolling() {
        let mut gpu = Gpu::new();

        // init GPU
        gpu.background_display_enabled = true;
        gpu.background_tile_data_area = true;
        gpu.background_tile_map_area = TileMapArea::X9C00;

        // init VRAM
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x0200, 0x80);
        gpu.write_vram(0x0201, 0x80);
        gpu.write_vram(0x0210, 0x80);
        gpu.write_vram(0x0211, 0x80);

        // set tile map
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x1C20, 0x20);
        gpu.write_vram(0x1C21, 0x21);

        // scroll on y axis and draw the line
        gpu.viewport_y_offset = 1;
        gpu.viewport_x_offset = 0;
        gpu.current_line = 7; // line 8 now corresponds to line 9
        gpu.draw_line();

        // check frame buffer
        // line 9 * 160 = 1440 / 0x05A0
        assert_eq!(gpu.frame_buffer[0x05A0], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x05A8], PixelColor::BLACK as u8);

        // scroll on x axis and draw the line
        gpu.viewport_y_offset = 0;
        gpu.viewport_x_offset = 1;
        gpu.current_line = 8;
        gpu.draw_line();

        // check frame buffer
        // line 8 * 160 = 1280 / 0x0500
        assert_eq!(gpu.frame_buffer[0x0507], PixelColor::BLACK as u8);
    }

    #[test]
    fn test_draw_frame() {
        let mut gpu = Gpu::new();
        let mut nvic = Nvic::new();

        // init GPU
        gpu.background_display_enabled = true;
        gpu.background_tile_data_area = true;
        gpu.background_tile_map_area = TileMapArea::X9800;
        gpu.lcd_display_enabled = true;

        // init VRAM
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x0200, 0x80);
        gpu.write_vram(0x0201, 0x80);

        // set tile map
        // here we're looking for tile at index 0 and 1
        gpu.write_vram(0x1800, 0x20);
        gpu.write_vram(0x1801, 0x20);
        // here we're looking for tile at index 32 and 33
        gpu.write_vram(0x1820, 0x20);
        gpu.write_vram(0x1821, 0x20);
        // here we're looking for tile map at index 512 and 513 / line 129
        gpu.write_vram(0x1A00, 0x20);
        gpu.write_vram(0x1A01, 0x20);

        // draw the line in the frame buffer
        while gpu.current_line < SCREEN_HEIGHT as u8 {
            gpu.run(1, &mut nvic);
        }

        // check frame buffer
        // line 0 * 160 = 0 / 0x0000
        assert_eq!(gpu.frame_buffer[0x0000], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0008], PixelColor::BLACK as u8);
        // line 8 * 160 = 1280 / 0x0500
        assert_eq!(gpu.frame_buffer[0x0500], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x0508], PixelColor::BLACK as u8);
        // line 128 * 160 = 20480 / 0x5000
        assert_eq!(gpu.frame_buffer[0x5000], PixelColor::BLACK as u8);
        assert_eq!(gpu.frame_buffer[0x5008], PixelColor::BLACK as u8);
    }

    #[test]
    fn test_vblank_interrupts() {
        let mut gpu = Gpu::new();
        let mut nvic = Nvic::new();

        gpu.lcd_display_enabled = true;

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::VBLANK, true);

        let mut runned_cycles: u32 = 0;

        // run GPU
        while runned_cycles < (SCREEN_HEIGHT * (ONE_LINE_CYCLES as usize) + 1) as u32 {
            gpu.run(1, &mut nvic);
            runned_cycles += 1;
        }

        // check that we are in vblank mode and vblank interrupt has been asserted
        assert_eq!(gpu.mode, GpuMode::VerticalBlank);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::VBLANK);
    }

    #[test]
    fn test_stat_interrupts() {
        let mut gpu = Gpu::new();
        let mut nvic = Nvic::new();

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::STAT, true);
        gpu.oam_interrupt_enabled = true;
        gpu.hblank_interrupt_enabled = true;
        gpu.vblank_interrupt_enabled = false;
        gpu.lcd_display_enabled = true;

        let mut runned_cycles: u32 = 0;

        // run GPU
        while runned_cycles < (OAM_SCAN_CYCLES + DRAW_PIXEL_CYCLES + 1) as u32 {
            gpu.run(1, &mut nvic);
            runned_cycles += 1;
        }

        // check that we are in vblank mode and vblank interrupt has been asserted
        assert_eq!(gpu.mode, GpuMode::HorizontalBlank);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::STAT);

        // run GPU
        runned_cycles = 0;
        while runned_cycles < (HORIZONTAL_BLANK_CYCLES) as u32 {
            gpu.run(1, &mut nvic);
            runned_cycles += 1;
        }

        // check that we are in vblank mode and vblank interrupt has been asserted
        assert_eq!(gpu.mode, GpuMode::OAMScan);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::STAT);
        assert_eq!(nvic.get_interrupt(), None);
    }

    #[test]
    fn test_vblank_stat_interrupts() {
        let mut gpu = Gpu::new();
        let mut nvic = Nvic::new();

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::STAT, true);
        gpu.oam_interrupt_enabled = false;
        gpu.hblank_interrupt_enabled = false;
        gpu.vblank_interrupt_enabled = true;
        gpu.lcd_display_enabled = true;

        let mut runned_cycles: u32 = 0;

        // run the gpu
        while runned_cycles < (SCREEN_HEIGHT * (ONE_LINE_CYCLES as usize) + (HORIZONTAL_BLANK_CYCLES as usize) + 1) as u32 {
            gpu.run(1, &mut nvic);
            runned_cycles += 1;
        }

        // check that we are in vblank mode and vblank interrupt has been asserted
        assert_eq!(gpu.mode, GpuMode::VerticalBlank);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::STAT);
    }

    #[test]
    fn test_compare_line() {
        let mut gpu = Gpu::new();
        let mut nvic = Nvic::new();

        nvic.master_enable(true);
        nvic.enable_interrupt(InterruptSources::STAT, true);
        gpu.line_compare_it_enable = true;
        gpu.compare_line = 2;
        gpu.lcd_display_enabled = true;

        assert_eq!(gpu.line_compare_state, false);

        let mut runned_cycles: u32 = 0;

        // run the gpu for 3 lines
        while runned_cycles < (ONE_LINE_CYCLES * 2 + HORIZONTAL_BLANK_CYCLES) as u32 {
            gpu.run(1, &mut nvic);
            runned_cycles += 1;
        }

        // check that we are in vblank mode and vblank interrupt has been asserted
        assert_eq!(gpu.current_line, 2);
        assert_eq!(gpu.line_compare_state, true);
        assert_eq!(nvic.get_interrupt().unwrap(), InterruptSources::STAT);
    }

    #[test]
    fn test_control_reg() {
        let mut gpu = Gpu::new();

        gpu.control_from_byte(0xDB);
        let reg = gpu.control_to_byte();
        
        assert_eq!(reg, 0xDB);
    }

    #[test]
    fn test_status_reg() {
        let mut gpu = Gpu::new();

        gpu.status_from_byte(0xDF);
        let reg = gpu.status_to_byte();
        
        assert_eq!(reg, 0xD8);
    }

    #[test]
    fn test_set_background_palette() {
        let mut gpu = Gpu::new();

        gpu.set_background_palette(0b10010011);

        assert_eq!(gpu.background_palette.color_3, PixelColor::DARK_GRAY);
        assert_eq!(gpu.background_palette.color_2, PixelColor::LIGHT_GRAY);
        assert_eq!(gpu.background_palette.color_1, PixelColor::WHITE);
        assert_eq!(gpu.background_palette.color_0, PixelColor::BLACK);
    }

    #[test]
    fn test_set_object_palette() {
        let mut gpu = Gpu::new();

        gpu.set_object_palette_0(0b10010011);

        assert_eq!(gpu.object_palette_0.color_3, PixelColor::DARK_GRAY);
        assert_eq!(gpu.object_palette_0.color_2, PixelColor::LIGHT_GRAY);
        assert_eq!(gpu.object_palette_0.color_1, PixelColor::WHITE);
        assert_eq!(gpu.object_palette_0.color_0, PixelColor::BLACK);

        gpu.set_object_palette_1(0b11010010);

        assert_eq!(gpu.object_palette_1.color_3, PixelColor::BLACK);
        assert_eq!(gpu.object_palette_1.color_2, PixelColor::LIGHT_GRAY);
        assert_eq!(gpu.object_palette_1.color_1, PixelColor::WHITE);
        assert_eq!(gpu.object_palette_1.color_0, PixelColor::DARK_GRAY);
    }

    #[test]
    fn test_draw_big_sprite() {
        let mut gpu = Gpu::new();

        // init GPU
        gpu.object_display_enabled = true;
        gpu.object_size = ObjectSize::OS8X16;

        // init VRAM
        // here we're looking for tile at index 0
        gpu.write_vram(0x0010, 0x7F);
        gpu.write_vram(0x0011, 0xFF);
        gpu.write_vram(0x0012, 0xFF);
        gpu.write_vram(0x0013, 0xFE);

        // set OAM
        // set y position
        gpu.write_oam(0x0000, 0x10);
        // set x position
        gpu.write_oam(0x0001, 0x08);
        // set tile index
        gpu.write_oam(0x0002, 0x01);
        // set attributes
        gpu.write_oam(0x0003, 0x00);

        // draw the line in the frame buffer
        gpu.current_line = 0;
        gpu.draw_line();
        gpu.current_line = 1;
        gpu.draw_line();

        // check frame buffer
        // line 1 * 160 = 160 / 0x00A0
        assert_eq!(gpu.frame_buffer[0x0000], PixelColor::DARK_GRAY as u8);
        assert_eq!(gpu.frame_buffer[0x00A7], PixelColor::LIGHT_GRAY as u8);

        // shift sprite y position for 31 lines down
        // set y position
        gpu.write_oam(0x0000, 0x2F);

        // draw the line in the frame buffer
        gpu.current_line = 31;
        gpu.draw_line();
        gpu.current_line = 32;
        gpu.draw_line();

        // check frame buffer
        // line 31 * 160 = 4960 / 0x1360
        assert_eq!(gpu.frame_buffer[0x1360], PixelColor::DARK_GRAY as u8);
        // line 32 * 160 = 5120 / 0x1400
        assert_eq!(gpu.frame_buffer[0x1407], PixelColor::LIGHT_GRAY as u8);

        // shift sprite x position for 3 columns right
        // set y position
        gpu.write_oam(0x0001, 0x0B);

        // draw the line in the frame buffer
        gpu.current_line = 31;
        gpu.draw_line();
        gpu.current_line = 32;
        gpu.draw_line();

        // check frame buffer
        // line 31 * 160 = 4960 / 0x1360
        assert_eq!(gpu.frame_buffer[0x1363], PixelColor::DARK_GRAY as u8);
        // line 32 * 160 = 5120 / 0x1400
        assert_eq!(gpu.frame_buffer[0x140A], PixelColor::LIGHT_GRAY as u8);
    }

    #[test]
    fn test_draw_window() {
        let mut gpu = Gpu::new();

        // init GPU
        gpu.window_display_enabled = true;
        gpu.background_display_enabled = true;
        gpu.window_tile_map_area = TileMapArea::X9800;
        gpu.background_tile_data_area = true;
        gpu.current_line = 0; // first line of the second tile row

        // init VRAM
        // here we're looking for tile at index 0
        gpu.write_vram(0x0000, 0x01);
        gpu.write_vram(0x0001, 0x00);

        // set tile map
        // here we're looking for tile at index 0
        gpu.write_vram(0x1800, 0x00);

        // draw the line in the frame buffer
        gpu.window_x_offset = 0;
        gpu.draw_line();
        assert_eq!(gpu.frame_buffer[0x0000], PixelColor::LIGHT_GRAY as u8);

        gpu.window_x_offset = 128;
        gpu.draw_line();
        assert_eq!(gpu.frame_buffer[0x0080], PixelColor::LIGHT_GRAY as u8);
    }
}