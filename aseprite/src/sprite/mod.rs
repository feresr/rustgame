use crate::read;
use std::fs::{self, File};
use std::io::{Read, Write};
use aseprite_loader::loader::AsepriteFile;

/**
* Fer is a simpler aseprite file format it includes the png data as part of the binary.
*/
#[derive(Debug, serde::Serialize)]
pub struct Frame {
    pub duration: u16,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, serde::Serialize)]
pub struct Slice {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub pivot_x: i32,
    pub pivot_y: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct Tag {
    pub name: String,
    pub from: u16,
    pub to: u16,
}

#[derive(serde::Serialize)]
pub struct Sprite {
    pub frames: Vec<Frame>,
    pub slices: Vec<Slice>,
    pub tags: Vec<Tag>,
}

impl Sprite {
    pub fn decode(file: &mut fs::File) -> Self {
        let mut frames = Vec::new();
        let mut slices = Vec::new();
        let mut tags = Vec::new();

        let frame_count = read!(u32, file);
        dbg!(frame_count);
        for _ in 0..frame_count {
            let duration = read!(u16, file);
            let x = read!(i16, file);
            let y = read!(i16, file);
            let width = read!(u16, file);
            let height = read!(u16, file);
            let frame = Frame {
                duration,
                x,
                y,
                width,
                height,
            };
            // Add the frame to the frames vector
            frames.push(frame);
        }

        let slice_count = read!(u32, file);
        dbg!(slice_count);
        for _ in 0..slice_count {
            let name_len = read!(u32, file);
            let mut name = vec![0u8; name_len as usize];
            file.read_exact(&mut name).unwrap();
            let name = String::from_utf8(name).unwrap();

            dbg!(&name);

            let x = read!(i32, file);
            let y = read!(i32, file);
            let width = read!(u32, file);
            let height = read!(u32, file);
            let pivot_x = read!(i32, file);
            let pivot_y = read!(i32, file);

            let slice = Slice {
                name,
                x,
                y,
                width,
                height,
                pivot_x,
                pivot_y,
            };
            // Add the slice to the slices vector
            slices.push(slice);
        }

        let tag_count = read!(u32, file);
        dbg!(tag_count);
        for _ in 0..tag_count {
            let name_len = read!(u32, file);
            let mut name = vec![0u8; name_len as usize];
            file.read_exact(&mut name).unwrap();
            let name = String::from_utf8(name).unwrap();
            dbg!(&name);

            let from = read!(u16, file);
            dbg!(from);
            let to = read!(u16, file);
            dbg!(to);
            let tag = Tag { name, from, to };
            // Add the tag to the tags vector
            tags.push(tag);
        }

        Sprite {
            frames,
            slices,
            tags,
        }
    }

    pub fn encode(aseprite: &AsepriteFile, packing_width: usize, packing_height: usize) {
        // Initialize a new Fer instance
        let mut output = File::create("output.bin").unwrap();

        let mut packing_x = 0;
        let mut packing_y = 0;

        output
            .write_all(&(aseprite.frames.len() as u32).to_le_bytes())
            .unwrap();
        for frame in aseprite.frames.iter() {
            for cel in &frame.cels {
                output.write_all(&frame.duration.to_le_bytes()).unwrap();
                dbg!(&frame.duration);
                if let Some(image) = &mut aseprite.images.get(cel.image_index) {
                    let x = packing_x * aseprite.file.header.width;
                    let y = packing_y * aseprite.file.header.height;

                    output.write_all(&x.to_le_bytes()).unwrap();
                    output.write_all(&y.to_le_bytes()).unwrap();
                    output
                        .write_all(&aseprite.file.header.width.to_le_bytes())
                        .unwrap();
                    output
                        .write_all(&aseprite.file.header.height.to_le_bytes())
                        .unwrap();

                    if (packing_x as usize) < packing_width - 1 {
                        packing_x += 1;
                    } else {
                        packing_x = 0;
                        packing_y += 1;
                    }
                    break;
                }
            }
        }

        let slices = &aseprite.file.slices;
        output
            .write_all(&(slices.len() as u32).to_le_bytes())
            .unwrap();
        for slice in &mut slices.iter() {
            output
                .write_all(&(slice.name.len() as u32).to_le_bytes())
                .unwrap();
            output.write_all(slice.name.as_bytes()).unwrap();

            let key = slice.slice_keys.first().unwrap();
            output.write_all(&key.x.to_le_bytes()).unwrap();
            output.write_all(&key.y.to_le_bytes()).unwrap();
            output.write_all(&key.width.to_le_bytes()).unwrap();
            output.write_all(&key.height.to_le_bytes()).unwrap();
            if let Some(p) = &key.pivot {
                output.write_all(&p.x.to_le_bytes()).unwrap();
                output.write_all(&p.y.to_le_bytes()).unwrap();
            } else {
                panic!("Pivot is missing")
            }
        }
        output
            .write_all(&(aseprite.tags.len() as u32).to_le_bytes())
            .unwrap();
        for tag in &mut aseprite.tags.iter() {
                output
                    .write_all(&(tag.name.len() as u32).to_le_bytes())
                    .unwrap();
                output.write_all(&tag.name.as_bytes()).unwrap();
                output.write_all(&tag.range.start().to_le_bytes()).unwrap();
                output.write_all(&tag.range.end().to_le_bytes()).unwrap();
        }
    }
}
