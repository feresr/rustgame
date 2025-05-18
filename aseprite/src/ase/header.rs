use crate::{BYTE, DWORD, SHORT, WORD};

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    pub file_size: DWORD,
    pub magic_number: WORD,
    pub frames: WORD,
    pub width: WORD,
    pub height: WORD,
    pub color_depth: WORD,
    pub flags: DWORD,
    pub speed: WORD,
    pub zero_0: DWORD,
    pub zero_1: DWORD,
    pub palette_transparent_index: BYTE,
    pub ignore: [BYTE; 3],
    pub color_count: WORD,
    pub pixel_width: BYTE,
    pub pixel_height: BYTE,
    pub grid_x: SHORT,
    pub grid_y: SHORT,
    pub grid_width: WORD,
    pub grid_height: WORD,
    pub future: [BYTE; 84],
}