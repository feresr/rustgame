use std::{fs, io::Read};

use crate::{read, BYTE, DWORD, SHORT, WORD};

#[derive(Debug)]
pub struct Tileset {
    pub id: DWORD,
    pub flags: DWORD,
    pub count: DWORD,
    pub tile_width: WORD,
    pub tile_height: WORD,
    pub base_index: SHORT,
    pub name: String,
    pub external_file: Option<ExternalFile>,
    pub tileset_image: Option<CompressedTilesetImage>,
}

impl Tileset {
    pub fn new(file: &mut fs::File) -> Self {
        let id = read!(DWORD, file);
        let flags = read!(DWORD, file);
        let count = read!(DWORD, file);
        let tile_width = read!(WORD, file);
        let tile_height = read!(WORD, file);
        let base_index = read!(SHORT, file);

        let mut buffer = [0u8; 14];
        file.read_exact(&mut buffer).unwrap();

        let name_length = read!(WORD, file);
        let mut buffer = vec![0u8; name_length as usize];
        file.read_exact(&mut buffer).unwrap();
        let name = String::from_utf8(buffer).unwrap();

        let mut external_file = None;
        if flags & 0b1 != 0 {
            external_file = Some(ExternalFile::new(file));
        }
        let mut tileset_image = None;
        if flags & 0b10 != 0 {
            // TODO:: tileset_image
            tileset_image = Some(CompressedTilesetImage::new(file));
        }

        Tileset {
            id,
            flags,
            count,
            tile_width,
            tile_height,
            base_index,
            name,
            external_file,
            tileset_image,
        }
    }
}

#[derive(Debug)]
pub struct ExternalFile {
    pub id: DWORD,
    pub tileset_id: DWORD,
}

impl ExternalFile {
    pub fn new(file: &mut fs::File) -> Self {
        let id = read!(DWORD, file);
        let tileset_id = read!(DWORD, file);
        ExternalFile { id, tileset_id }
    }
}

#[derive(Debug)]
pub struct CompressedTilesetImage {
    pub compressed_data_length: DWORD,
    pub pixels: Vec<BYTE>, // Could be compressed
}
impl CompressedTilesetImage {
    pub fn new(file: &mut fs::File) -> Self {
        let compressed_data_length = read!(DWORD, file);
        let mut buffer = vec![0u8; compressed_data_length as usize];
        file.read_to_end(&mut buffer).unwrap();
        CompressedTilesetImage {
            compressed_data_length,
            pixels: buffer,
        }
    }
}
