use std::fs::{self};
use aseprite_loader::loader::AsepriteFile;

#[derive(serde::Serialize)]
pub struct TileMap {
}

impl TileMap {
    pub fn decode(file: &mut fs::File) -> Self {
        TileMap {}
    }

    pub fn encode(aseprite: &mut AsepriteFile, packing_width: usize, packing_height: usize) {

    }
}
