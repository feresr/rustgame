#[cfg(test)]
mod test;

pub mod chunks;
pub mod frame;
mod header;

use std::{
    cmp, fs,
    io::{Read, Seek, SeekFrom, Write},
};

use std::fmt::Debug;

use chunks::{
    cel::Cel, color_profile::ColorProfile, layer::Layer, palette::{OldPaletteChunk, Palette}, slice::Slice, tag::{Tag, Tags}, Chunk
};
use frame::Frame;
use header::Header;

type BYTE = u8;
type WORD = u16;
type SHORT = i16;
type LONG = i32;
type DWORD = u32;
type FIXED = i32;

#[derive(Debug)]
pub struct Aseprite {
    pub header : Header,
    pub frame_count: u32,
    pub frames: Vec<Frame>,
}

#[macro_export]
macro_rules! read {
    ($t:ty, $file:expr) => {{
        let mut buffer = [0u8; std::mem::size_of::<$t>()];
        $file.read_exact(&mut buffer).unwrap();
        <$t>::from_le_bytes(buffer)
    }};
}

#[allow(dead_code)]
impl Aseprite {
    pub fn new(file_path: &str) -> Self {
        
        let mut file = fs::File::open(file_path).expect("Failed to open binary file");
        // Can't use std::mem::size_of::<Header>() because of memory padding
        let mut buf = [0u8; 128];
        file.read_exact(&mut buf).unwrap();

        let header: Header = unsafe { std::ptr::read(buf.as_ptr() as *const _) };
        assert!(header.magic_number == 0xA5E0);
        
        let mut frames: Vec<Frame> = Vec::with_capacity(header.frames as usize);
        for _ in 0..header.frames {
            let mut frame: Frame = Frame::new(&mut file);
            let chunk_count: DWORD = cmp::max(frame.chunk_count_legacy as DWORD, frame.chunk_count);
            for _ in 0..chunk_count {
                let mut dword = [0u8; 4];
                let mut word = [0u8; 2];
                file.read_exact(&mut dword).unwrap();
                let chunk_size = u32::from_le_bytes(dword);
                file.read_exact(&mut word).unwrap();
                let chunk_type = u16::from_le_bytes(word);
                // -6 because we alre
                let chunk_data_size = chunk_size - 6;

                match chunk_type {
                    0x2007 => {
                        // Color profile chunk
                        frame.color_profile = Some(ColorProfile::new(&mut file));
                    }
                    0x2019 => {
                        // Palette chunk
                        let palette = Palette::new(&mut file);
                            frame.palettes.push(palette);
                    }
                    0x0004 => {
                        OldPaletteChunk::read_pass(&mut file);
                        continue;
                    }
                    0x0011 => {
                        OldPaletteChunk::read_pass(&mut file);
                        continue;
                    }
                    0x2004 => { 
                        let layer = Layer::new(&mut file);
                        frame.layers.push(layer);
                    }
                    0x2005 => { 
                        let cel = Cel::new(&mut file, header.color_depth, chunk_data_size);
                        frame.cels.push(cel);
                    }
                    0x2018 => { 
                        let tag = Tags::new(&mut file);
                        frame.tags = Some(tag);
                    },
                    0x2022 => {
                        // Slice chunk
                        let c = Slice::new(&mut file);
                        frame.slices.push(c);
                    }
                    0x2020 => {
                        // User data chunk
                        file.seek(SeekFrom::Current(chunk_data_size as i64))
                            .unwrap();
                        continue;
                    }
                    0x2023 => {
                        // Tileset chunk
                        panic!("Tileset chunk not implemented")
                    }
                    _ => {
                        // Unknown chunk
                        panic!("Unkown chunk not implemented {}", chunk_type)
                    }
                };
            }
            frames.push(frame);
        }

        // let slices: Vec<Slice> = Self::read_slices(&mut file);
        // let tags: Vec<Tag> = Self::read_tags(&mut file);

        Self {
            frame_count: frames.len() as u32,
            frames,
            header
        }
    }
}