use std::{fs, io::Read};

use crate::sprite::Sprite;
use crate::tilemap::TileMap;

#[test]
fn decode_sprite_test() {
    let mut file = fs::File::open("output.bin").unwrap();
    let sprite = Sprite::decode(&mut file);

    let decoded = fs::File::create("output.md").unwrap();
    serde_json::to_writer_pretty(&decoded, &sprite).unwrap();
}

#[test]
fn decode_tilemap_test() {
    let mut file = fs::File::open("output.bin").unwrap();
    let tilemap = TileMap::decode(&mut file);

    let decoded = fs::File::create("output.md").unwrap();
    serde_json::to_writer_pretty(&decoded, &tilemap).unwrap();
}
