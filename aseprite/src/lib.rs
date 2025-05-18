extern crate core;

use std::io::Read;
use flate2::read::ZlibDecoder;

// pub mod ase;
#[cfg(test)]
mod decode_test;
#[cfg(test)]
mod encode_test;
pub mod sprite;
pub mod tilemap;

type BYTE = u8;
type WORD = u16;
type SHORT = i16;
type LONG = i32;
type DWORD = u32;
type FIXED = i32;

#[macro_export]
macro_rules! read {
    ($t:ty, $file:expr) => {{
        let mut buffer = [0u8; std::mem::size_of::<$t>()];
        $file.read_exact(&mut buffer).unwrap();
        <$t>::from_le_bytes(buffer)
    }};
}

pub fn uncompress(compressed: &[u8]) -> Vec<u8> {
        let mut decoder = ZlibDecoder::new(compressed);
        let mut uncompressed_data = Vec::new();
        decoder.read_to_end(&mut uncompressed_data).unwrap();
        uncompressed_data
}
