use std::{fs, io::Read};

use crate::{read, BYTE, DWORD, WORD};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Layer {
    pub flags: WORD,
    pub kind: WORD,
    pub child_level: WORD,
    pub blend_mode: WORD,
    pub opacity: BYTE,
    pub reserved: [BYTE; 3],
    pub name: String,
    pub tileset_index: Option<DWORD>,
    pub uuid: Option<[BYTE; 16]>,
}

impl Layer {
    pub fn new(file: &mut fs::File) -> Self {
        let flags = read!(WORD, file);
        let kind = read!(WORD, file);
        let child_level = read!(WORD, file);
        let _ = read!(WORD, file); // pixel_width (ignored, according to the specification)
        let _ = read!(WORD, file); // pixel_height (ignored, according to the specification)
        let blend_mode = read!(WORD, file);
        let opacity = read!(BYTE, file);
        let reserved = [read!(BYTE, file), read!(BYTE, file), read!(BYTE, file)];

        let name_lenght = read!(WORD, file);
        dbg!(name_lenght);
        let mut buffer = vec![0u8; name_lenght as usize];
        file.read_exact(&mut buffer).unwrap();
        let name = String::from_utf8(buffer).unwrap();
        
        Layer {
            flags,
            kind,
            child_level,
            blend_mode,
            opacity,
            reserved,
            name,
            tileset_index: None,
            uuid: None,
        }
    }
}