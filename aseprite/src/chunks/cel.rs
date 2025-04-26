use std::{fs, io::Read};

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
    pub image_data: Option<ImageData>,                 // For kind = 0 and 2
    pub frame_position_link: Option<WORD>,             // Find kind 1
    pub compressed_tilemap: Option<CompressedTileMap>, // Find kind 3
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ImageData {
    pub color_depth : WORD,
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
    reserved: [BYTE; 10],
    compressed_tiles: Vec<BYTE>,
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
            
            // if kind == 2 {
            //     // data is compressed using zlib
            //     let mut uncompressed: Vec<BYTE> = Vec::new();
            //     let mut decodder = ZlibDecoder::new(&pixels[..]);
            //     decodder.read_to_end(&mut uncompressed).unwrap();
            //     pixels = uncompressed;
            // }
            Some(ImageData {
                color_depth,
                width: w,
                height: h,
                data: pixels,
                is_zlib_compressed
            })
        } else {
            None
        };

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
