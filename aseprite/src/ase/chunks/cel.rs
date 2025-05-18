use std::{
    fs,
    io::{Read, Seek},
};

use flate2::read::{ZlibDecoder, ZlibEncoder};

use crate::{read, BYTE, DWORD, SHORT, WORD};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Cel {
    pub layer_index: WORD,
    pub position_x: SHORT,
    pub position_y: SHORT,
    pub opacity: BYTE,
    pub kind: WORD,
    pub z_index: SHORT,
    pub reserved: [BYTE; 5],
    pub image_data: Option<ImageData>,     // For kind = 0 and 2
    pub frame_position_link: Option<WORD>, // Find kind 1
    pub compressed_tilemap: Option<CompressedTileMap>, // Find kind 3
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ImageData {
    pub color_depth: WORD,
    pub width: WORD,
    pub height: WORD,
    pub data: Vec<BYTE>, // Could be compressed
    pub is_zlib_compressed: bool,
}
impl ImageData {
    pub fn uncompress(&self) -> Vec<u8> {
        if self.is_zlib_compressed {
            let mut decoder = ZlibDecoder::new(&self.data[..]);
            let mut uncompressed_data = Vec::new();
            decoder.read_to_end(&mut uncompressed_data).unwrap();
            uncompressed_data
        } else {
            panic!("Image data is not compressed");
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CompressedTileMap {
    tiles_width: WORD,
    tiles_height: WORD,
    bits_per_tile: WORD, // always 32
    tile_id_bitmask: DWORD,
    flip_x_bitmask: DWORD,
    flip_y_bitmask: DWORD,
    flip_diagonal_bitmask: DWORD,
    compressed_tiles: Vec<BYTE>,
}

impl CompressedTileMap {
    fn new(file: &mut fs::File, remaining_size : u32) -> Self {
        let tiles_width = read!(WORD, file);
        let tiles_height = read!(WORD, file);
        let bits_per_tile = read!(WORD, file);
        let tile_id_bitmask = read!(DWORD, file);
        let flip_x_bitmask = read!(DWORD, file);
        let flip_y_bitmask = read!(DWORD, file);
        let flip_diagonal_bitmask = read!(DWORD, file);

        file.seek(std::io::SeekFrom::Current(10)).unwrap();

        let mut pixels = vec![0u8; remaining_size as usize];
        file.read_exact(&mut pixels).unwrap();

        CompressedTileMap { tiles_width, tiles_height, bits_per_tile, tile_id_bitmask, flip_x_bitmask, flip_y_bitmask, flip_diagonal_bitmask, compressed_tiles : pixels }
    }
}

impl Cel {
    // WORD        Color depth (bits per pixel) (read from header)
    // 32 bpp = RGBA
    // 16 bpp = Grayscale
    // 8 bpp = Indexed
    pub fn new(file: &mut fs::File, color_depth: WORD, chunk_size: u32) -> Self {
        let mut remaining_size = chunk_size;
        let layer_index = read!(WORD, file);
        remaining_size -= 2;
        let position_x = read!(SHORT, file);
        remaining_size -= 2;
        let position_y = read!(SHORT, file);
        remaining_size -= 2;
        let opacity = read!(BYTE, file);
        remaining_size -= 1;
        let kind = read!(WORD, file);
        remaining_size -= 2;
        let z_index = read!(SHORT, file);
        remaining_size -= 2;
        let mut reserved = [0u8; 5];
        remaining_size -= 5;
        file.read(&mut reserved).unwrap();

        let image_data = if kind == 0 || kind == 2 {
            // Raw image data
            let w = read!(WORD, file);
            remaining_size -= 2;
            let h = read!(WORD, file);
            remaining_size -= 2;
            let mut pixels = vec![0u8; remaining_size as usize];
            file.read_exact(&mut pixels).unwrap();

            let is_zlib_compressed = kind == 2;

            Some(ImageData {
                color_depth,
                width: w,
                height: h,
                data: pixels,
                is_zlib_compressed,
            })
        } else {
            None
        };

        if kind == 3 {
            CompressedTileMap::new(file, remaining_size);
        }

        Cel {
            layer_index,
            position_x,
            position_y,
            opacity,
            kind,
            z_index,
            reserved: [0; 5],
            image_data,
            frame_position_link: None, // TODO
            compressed_tilemap: None,  // TODO
        }
    }
}
