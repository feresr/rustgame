use std::{fs, io::Read};

use crate::{chunks::{cel::Cel, color_profile::ColorProfile, layer::Layer, palette::Palette, slice::Slice, tag::Tags}, read, BYTE, DWORD, WORD};

#[repr(C)]
#[derive(Debug)]
pub struct Frame {
    pub bytes: DWORD,             // 4
    pub magic_number: WORD,       // 2
    pub chunk_count_legacy: WORD, // 2
    pub duration: WORD,           // 2
    pub zero: [BYTE; 2],          // 2
    pub chunk_count: DWORD,       // 4
    pub cels : Vec<Cel>,
    pub palettes : Vec<Palette>,
    pub layers : Vec<Layer>,
    pub tags : Option<Tags>,
    pub color_profile : Option<ColorProfile>,
    pub slices : Vec<Slice>,
}

impl Frame {
    pub fn new(file: &mut fs::File) -> Self {
        let bytes = read!(DWORD, file);
        let magic_number = read!(WORD, file);
        assert!(magic_number == 0xF1FA);
        let chunk_count_legacy = read!(WORD, file);
        let duration = read!(WORD, file);
        let zero = [read!(BYTE, file), read!(BYTE, file)];
        let chunk_count = read!(DWORD, file);

        Self {
            bytes,
            magic_number,
            chunk_count_legacy,
            duration,
            zero,
            chunk_count,
            layers: Vec::new(),
            cels: Vec::new(),
            palettes: Vec::new(),
            tags: None,
            color_profile: Option::None,
            slices: Vec::new(),
        }
    }
}
