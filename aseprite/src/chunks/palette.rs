use std::{fs, io::Read};

use crate::{BYTE, DWORD, WORD};


#[derive(Debug, Clone)]
pub struct Palette {
    pub size: DWORD,
    pub first_color_index: DWORD,
    pub last_color_index: DWORD,
    pub reserved: [BYTE; 8],
    pub entries: Vec<PaletteEntry>,
}

impl Palette {
    pub fn new(file: &mut fs::File) -> Self {
        let mut dword = [0u8; 4];

        file.read_exact(&mut dword).unwrap();
        let size = u32::from_le_bytes(dword);
        
        file.read_exact(&mut dword).unwrap();
        let first_color_index = u32::from_le_bytes(dword);
        
        file.read_exact(&mut dword).unwrap();
        let last_color_index = u32::from_le_bytes(dword);

        let mut reserved = [0u8; 8];
        file.read_exact(&mut reserved).unwrap();

        let mut entries = Vec::new();
        for _ in 0..size {
            let entry = PaletteEntry::new(file);
            entries.push(entry);
        }
        Palette {
            size,
            first_color_index,
            last_color_index,
            reserved,
            entries,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub has_name: WORD,
    pub red: BYTE,
    pub green: BYTE,
    pub blue: BYTE,
    pub alpha: BYTE,
    pub name: Option<String>,
}

impl PaletteEntry {
    fn new(file: &mut fs::File) -> Self {
        let mut word = [0u8; 2];
        let mut byte = [0u8; 1];
        file.read_exact(&mut word).unwrap();
        let has_name = u16::from_le_bytes(word);
        file.read_exact(&mut byte).unwrap();
        let red = byte[0];
        file.read_exact(&mut byte).unwrap();
        let green = byte[0];
        file.read_exact(&mut byte).unwrap();
        let blue = byte[0];
        file.read_exact(&mut byte).unwrap();
        let alpha = byte[0];

        let name = if has_name != 0 {
            let mut word = [0u8; 2];
            file.read_exact(&mut word).unwrap();
            let lenght = u16::from_le_bytes(word);
            dbg!(&lenght);
            let mut name = String::with_capacity(lenght as usize);
            let mut string_buffer = vec![0u8; lenght as usize];
            file.read_exact(&mut string_buffer).unwrap();
            name.push_str(std::str::from_utf8(&string_buffer).unwrap());
            Some(name)
        } else {
            None
        };
        PaletteEntry {
            has_name,
            red,
            green,
            blue,
            alpha,
            name,
        }
    }
}


#[derive(Debug)]
pub struct OldPaletteChunk { }
impl OldPaletteChunk {
    pub fn read_pass(file: &mut fs::File) {
        let mut word = [0u8; 2];
        file.read_exact(&mut word).unwrap();
        let packet_count = u16::from_le_bytes(word);
        for _ in 0..packet_count {
            let mut byte = [0u8; 1];
            file.read_exact(&mut byte).unwrap();
            let _ = byte[0]; // skip
            file.read_exact(&mut byte).unwrap();
            let colors = byte[0];
            for _ in 0..colors {
                // read 3 colors
                let mut byte = [0u8; 3];
                file.read_exact(&mut byte).unwrap();
            }
        }
    }
}
