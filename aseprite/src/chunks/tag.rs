use std::{fs, io::Read};

use crate::{read, BYTE, WORD};

#[derive(Debug)]
pub struct Tags {
    pub count: WORD,
    reserved: [u8; 8],
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub from: WORD,
    pub to: WORD,
    pub loop_animation_direcion: BYTE,
    pub repeat_count: WORD,
    reserved: [u8; 6],
    pub color: [u8; 3],
    pub zero: [u8; 1],
    pub name: String,
}

impl Tag {
    fn new(file : &mut fs::File) -> Self {
        let from = read!(WORD, file);
        let to = read!(WORD, file);
        let loop_animation_direcion = read!(BYTE, file);
        let repeat_count = read!(WORD, file);
        
        let mut reserved = [0u8; 6];
        file.read_exact(&mut reserved).unwrap();
        
        let color = [read!(BYTE, file), read!(BYTE, file), read!(BYTE, file)];
        
        let zero = [read!(BYTE, file)];
        
        let name_lenght = read!(WORD, file);
        let mut buffer = vec![0u8; name_lenght as usize];
        file.read_exact(&mut buffer).unwrap();
        let name = String::from_utf8(buffer).unwrap();

        Self {
            from,
            to,
            loop_animation_direcion,
            repeat_count,
            reserved,
            color,
            zero,
            name,
        }
    }
}

impl Tags {
    pub fn new(file: &mut fs::File) -> Self {
        let count = read!(WORD, file);

        let mut reserved = [0u8; 8];
        file.read_exact(&mut reserved).unwrap();
        
        let mut tags = vec![];
        for _ in 0..count {
            tags.push(Tag::new(file));
        }

        Self {
            count,
            reserved,
            tags,
        }
    }
}
