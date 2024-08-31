use std::{ffi::CStr, fs, io::Read};

#[derive(Debug)]
pub struct Frame {
    pub duration: u16,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub from: u16,
    pub to: u16,
}

#[derive(Debug)]
pub struct Slice {
    pub x: u16,
    pub y: u16,
    width: u16,
    height: u16,
    pub pivot_x: u16,
    pub pivot_y: u16,
}
#[derive(Debug)]
pub struct Aseprite {
    pub frame_count: u32,
    pub frames: Vec<Frame>,
    pub slices: Vec<Slice>,
    pub tags: Vec<Tag>,
}

/***
 * Simple util from loading (custom) aseprite binary files
 * File structure:
 *
 * 2 bytes for frame count
 * for each frame:
 *   2 bytes for duration
 *   2 bytes for x
 *   2 bytes for y
 *   2 bytes for width
 *   2 bytes for height
 * 2 bytes for number of slices
 *   for each slice:
 *     2 bytes for x
 *     2 bytes for y
 *     2 bytes for width
 *     2 bytes for height
 *     2 bytes for pivot x (defaults to 0,0)
 *     2 bytes for pivot y (defaults to 0,0)
 * 2 bytes for number of tags
 *  for each tag:
 *     a null terminated string for name
 *     2 bytes for 'from' frame
 *     2 bytes for 'to' frame
 */
impl Aseprite {
    fn read_tags(file: &mut fs::File) -> Vec<Tag> {
        let mut buffer = [0u8; 2]; // Buffer to read 16 bits (2 bytes) at a time
        file.read_exact(&mut buffer).unwrap();
        let tag_count = u16::from_le_bytes(buffer);
        let mut tags: Vec<Tag> = Vec::with_capacity(tag_count as usize);
        for _ in 0..tag_count {
            let mut name = String::new();
            loop {
                let mut char = [0u8; 1]; // (1 byte) at a time
                file.read_exact(&mut char).unwrap();
                let byte = char[0];
                if byte == 0 {
                    // null terminator
                    break;
                }
                name.push(byte as char);
            }

            file.read_exact(&mut buffer).unwrap();
            let from = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let to = u16::from_le_bytes(buffer);

            tags.push(Tag { name, from, to });
        }
        tags
    }

    fn read_slices(file: &mut fs::File) -> Vec<Slice> {
        let mut buffer = [0u8; 2]; // Buffer to read 16 bits (2 bytes) at a time
        file.read_exact(&mut buffer).unwrap();
        let slice_count = u16::from_le_bytes(buffer);

        let mut slices: Vec<Slice> = Vec::with_capacity(slice_count as usize);
        for i in 0..slice_count {
            file.read_exact(&mut buffer).unwrap();
            let x = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let y = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let w = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let h = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let pivot_x = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let pivot_y = u16::from_le_bytes(buffer);

            slices.push(Slice {
                x,
                y,
                width: w,
                height: h,
                pivot_x,
                pivot_y,
            });
        }
        slices
    }
    pub fn new(file_path: &str) -> Self {
        let mut file = fs::File::open(file_path).expect("Failed to open binary file");
        let mut buffer = [0u8; 2]; // Buffer to read 16 bits (2 bytes) at a time

        file.read_exact(&mut buffer).unwrap();
        let frame_count = u16::from_le_bytes(buffer);

        let mut frames: Vec<Frame> = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            file.read_exact(&mut buffer).unwrap();
            // TODO: Divide by 16.66666
            let duration = u16::from_le_bytes(buffer) / 16;
            file.read_exact(&mut buffer).unwrap();
            let x = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let y = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let w = u16::from_le_bytes(buffer);
            file.read_exact(&mut buffer).unwrap();
            let h = u16::from_le_bytes(buffer);

            frames.push(Frame {
                duration,
                x,
                y,
                width: w,
                height: h,
            });
        }

        let slices: Vec<Slice> = Self::read_slices(&mut file);
        let tags: Vec<Tag> = Self::read_tags(&mut file);

        Self {
            frame_count: frame_count as u32,
            frames,
            slices,
            tags,
        }
    }
}
