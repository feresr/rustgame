use crate::{read, DWORD, LONG, WORD};
use std::{fs, io::Read};

#[derive(Debug, Clone)]
pub struct Slice {
    key_count: DWORD,
    flags: DWORD,
    reserved: DWORD,
    pub name: String,
    pub keys: Vec<Key>,
}

impl Slice {
    pub fn new(file: &mut fs::File) -> Self {
        let key_count = read!(DWORD, file);
        let flags = read!(DWORD, file);
        let reserved = read!(DWORD, file);

        let name_lenght = read!(WORD, file);
        let mut buffer = vec![0u8; name_lenght as usize];
        file.read_exact(&mut buffer).unwrap();
        let name = String::from_utf8(buffer).unwrap();

        let mut keys = Vec::new();
        for _ in 0..key_count {
            keys.push(Key::new(file, flags));
        }

        Self {
            key_count,
            flags,
            reserved,
            name,
            keys,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Key {
    pub frame: DWORD,
    pub x: LONG,
    pub y: LONG,
    pub width: DWORD,
    pub height: DWORD,
    nine_patch: Option<NinePatch>,
    pub pivot: Option<Pivot>,
}
impl Key {
    pub fn new(file: &mut fs::File, flags: DWORD) -> Self {
        let frame = read!(DWORD, file);
        let x = read!(LONG, file);
        let y = read!(LONG, file);
        let width = read!(DWORD, file);
        let height = read!(DWORD, file);

        let nine_patch = if flags & 0b1 != 0 {
            Some(NinePatch {
                center_x: read!(LONG, file),
                center_y: read!(LONG, file),
                center_width: read!(DWORD, file),
                center_height: read!(DWORD, file),
            })
        } else {
            None
        };
        let pivot = if flags & 0b10 != 0 {
            Some(Pivot {
                x: read!(LONG, file),
                y: read!(LONG, file),
            })
        } else {
            None
        };
        Self {
            frame,
            x,
            y,
            width,
            height,
            nine_patch,
            pivot,
        }
    }
}

#[derive(Debug, Clone)]
struct NinePatch {
    center_x: LONG,
    center_y: LONG,
    center_width: DWORD,
    center_height: DWORD,
}

#[derive(Debug, Clone)]
pub struct Pivot {
    pub x: LONG,
    pub y: LONG,
}
