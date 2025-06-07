use std::{collections::HashMap, fs, rc::Rc};

use aseprite::sprite;
use aseprite::sprite::Sprite;
use engine::{
    audio::AudioTrack,
    graphics::{
        common::RectF,
        texture::{SubTexture, Texture},
    },
};
use ldtk_rust::Project;
use std::mem::size_of;

use crate::components::{room::{MapData, Room, SavedRoom}, sprite::Frame};
use crate::{
    components::sprite::{Animation, Tileset},
    game_state::GameState,
    MEMORY_PTR,
};
use crate::map::Map;

#[allow(dead_code)]
pub struct Content {
    pub tilesets: HashMap<i64, Tileset>,
    pub textures: HashMap<String, Rc<Texture>>,
    // animation sets
    sprites: HashMap<String, HashMap<String, Animation>>,
    pub tracks: HashMap<&'static str, AudioTrack>,
    pub map: Map,
}

impl Content {
    pub fn get() -> &'static mut Content {
        unsafe {
            let storage_ptr = (*MEMORY_PTR).storage.as_mut_ptr() as *mut GameState;
            let content = storage_ptr.add(size_of::<GameState>()) as *mut Content;
            &mut (*content)
        }
    }

    pub fn sprite(name: &str) -> &'static HashMap<String, Animation> {
        dbg!(name);
        &Content::get().sprites[name]
    }

    pub fn load(content_ptr: *mut Content) {
        // TODO: Async?
        let mut textures = HashMap::new();
        let mut sprites = HashMap::new();
        let mut tilesets = HashMap::new();

        let assets = fs::read_dir("game/src/assets/atlas/").unwrap();
        for asset in assets {
            let path = asset.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "json" {
                    continue;
                    //     if let Some(path_str) = path.to_str() {
                    //         // todo: Repalce .bin with .png
                    //         let png_str = path_str.replace(".bin", ".png");
                    //         let texture = Rc::new(Texture::from_path(&png_str));
                    //         let filename = path.file_stem().unwrap().to_str().unwrap();
                    //         textures.insert(filename.to_string(), texture);
                    //         let mut file = fs::File::open(path_str).unwrap();
                    //         let aseprite = Fer::new(&mut file); let texture = textures.get(filename).unwrap();
                    //         for slice in aseprite.slices.iter() {
                    //             let name: String = slice.name.clone();
                    //             let mut animations = HashMap::new();
                    //             let mut frames = Vec::new();
                    //             let frame = Frame {
                    //                 image: SubTexture::new(
                    //                     Rc::clone(texture),
                    //                     RectF {
                    //                         x: slice.x as f32,
                    //                         y: slice.y as f32,
                    //                         w: slice.width as f32,
                    //                         h: slice.height as f32,
                    //                     },
                    //                 ),
                    //                 duration: 1,
                    //                 pivot: ((slice.pivot_x) as u32, (slice.pivot_y) as u32),
                    //             };
                    //             frames.push(frame);
                    //             animations.insert(
                    //                 name.clone(),
                    //                 Animation {
                    //                     frames,
                    //                     name: name.clone(),
                    //                 },
                    //             );
                    //             sprites.insert(name.clone(), animations);
                    //         }
                    //     }
                }
            }
        }

        let assets = fs::read_dir("game/src/assets/").unwrap();
        for asset in assets {
            let path = asset.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "bin" {
                    if let Some(path_str) = path.to_str() {
                        // todo: Repalce .bin with .png
                        let png_str = path_str.replace(".bin", ".png");
                        let texture = Rc::new(Texture::from_path(&png_str));
                        let filename = path.file_stem().unwrap().to_str().unwrap();
                        textures.insert(filename.to_string(), texture);

                        let mut dotfer_file = fs::File::open(path_str).unwrap();
                        let dotfer = Sprite::decode(&mut dotfer_file);
                        let slice = dotfer.slices.first().unwrap();
                        let texture = textures.get(filename).unwrap();
                        let mut animations = HashMap::new();
                        for tag in dotfer.tags {
                            let mut frames = Vec::new();
                            let from = tag.from as usize;
                            let to = tag.to as usize;
                            let asset_parser_frame: &[sprite::Frame] = &dotfer.frames[from..=to];

                            for ap_frame in asset_parser_frame {
                                let frame = Frame {
                                    image: SubTexture::new(
                                        Rc::clone(texture),
                                        RectF {
                                            x: ap_frame.x as f32,
                                            y: ap_frame.y as f32,
                                            w: ap_frame.width as f32,
                                            h: ap_frame.height as f32,
                                        },
                                    ),
                                    duration: (ap_frame.duration as f32 / 16.66) as u32,
                                    pivot: (
                                        (slice.x + slice.pivot_x) as u32,
                                        (slice.y + slice.pivot_y) as u32,
                                    ),
                                };
                                frames.push(frame);
                            }
                            animations.insert(
                                tag.name.clone(),
                                Animation {
                                    frames,
                                    name: tag.name,
                                },
                            );
                        }
                        sprites.insert(filename.to_string(), animations);
                    }
                }

                if extension == "ldtk" {
                    let ldtk = Project::new(path);
                    for tileset_definition in ldtk.defs.tilesets {
                        let uid = tileset_definition.uid;
                        let tilset = Tileset::from_ldtk(tileset_definition);
                        // tilesets.insert(uid, tilset);
                        tilesets.insert(0, tilset);
                        break;
                    }
                }
            }
        }

        // TODO: Load all audio in folder
        let mut tracks = HashMap::new();
        let audio = AudioTrack::new("game/src/assets/audio/song.ogg").unwrap();
        tracks.insert("music-1", audio);
        let audio = AudioTrack::new("game/src/assets/audio/jump.ogg").unwrap();
        tracks.insert("jump", audio);
        let project = Project::new("game/src/assets/map.ldtk");
        // let project_ase = Aseprite::new("game/src/assets/map.ase");

        let mut map = Map::empty();
        let map_data = MapData::load();
        if let Some(data) = map_data {
            map.rooms = data.rooms.into_iter().map(|saved_room| Room::from(saved_room)).collect();
        }

        let content = Content {
            map, 
            tilesets,
            textures,
            sprites,
            tracks,
        };

        unsafe {
            content_ptr.write(content);
        }
    }

    pub fn map() -> &'static mut Map {
        &mut Content::get().map
    }
}
