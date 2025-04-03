use std::{collections::HashMap, fs, rc::Rc};

use engine::{
    audio::AudioTrack,
    graphics::{
        common::RectF,
        texture::{SubTexture, Texture},
    },
};
use ldtk_rust::Project;

use crate::{
    aseprite::{self, Aseprite},
    components::sprite::{Animation, Frame, Tileset},
    system::scene_system::Map,
};

#[allow(dead_code)]
pub struct Content {
    pub tilesets: HashMap<i64, Tileset>,
    pub textures: HashMap<String, Rc<Texture>>,
    // animation sets
    pub sprites: HashMap<String, HashMap<String, Animation>>,
    pub tracks: HashMap<&'static str, AudioTrack>,
    ldkt: Project,
    pub map: Map,
}

impl Content {
    pub fn load() -> Self {
        // TODO: Async?
        let mut textures = HashMap::new();
        let mut sprites = HashMap::new();
        let mut tilesets = HashMap::new();

        let assets = fs::read_dir("game/src/assets/atlas/").unwrap();
        for asset in assets {
            let path = asset.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "json" {
                    if let Some(path_str) = path.to_str() {
                        // todo: Repalce .bin with .png
                        let png_str = path_str.replace(".json", ".png");
                        let texture = Rc::new(Texture::from_path(&png_str));
                        let filename = path.file_stem().unwrap().to_str().unwrap();
                        textures.insert(filename.to_string(), texture);

                        let aseprite = Aseprite::new(path_str);
                        let texture = textures.get(filename).unwrap();

                        for slice in aseprite.slices.iter() {
                            let mut animations = HashMap::new();
                            let mut frames = Vec::new();
                            let frame = Frame {
                                image: SubTexture::new(
                                    Rc::clone(texture),
                                    RectF {
                                        x: slice.x as f32,
                                        y: slice.y as f32,
                                        w: slice.width as f32,
                                        h: slice.height as f32,
                                    },
                                ),
                                duration: 1,
                                pivot: ((slice.pivot_x) as u32, (slice.pivot_y) as u32),
                            };
                            frames.push(frame);
                            animations.insert(
                                slice.name.clone(),
                                Animation {
                                    frames,
                                    name: slice.name.clone(),
                                },
                            );
                            sprites.insert(slice.name.clone(), animations);
                        }
                    }
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

                        let aseprite = Aseprite::new(path_str);
                        let slice = aseprite.slices.first().unwrap();
                        let texture = textures.get(filename).unwrap();
                        let mut animations = HashMap::new();
                        for animation in aseprite.tags {
                            let mut frames = Vec::new();
                            let from = animation.from as usize;
                            let to = animation.to as usize;
                            let frame_slice: &[aseprite::Frame] = &aseprite.frames[from..=to];

                            for frame in frame_slice {
                                let frame = Frame {
                                    image: SubTexture::new(
                                        Rc::clone(texture),
                                        RectF {
                                            x: frame.x as f32,
                                            y: frame.y as f32,
                                            w: frame.width as f32,
                                            h: frame.height as f32,
                                        },
                                    ),
                                    duration: frame.duration as u32,
                                    pivot: (
                                        (slice.x + slice.pivot_x) as u32,
                                        (slice.y + slice.pivot_y) as u32,
                                    ),
                                };
                                frames.push(frame);
                            }
                            animations.insert(
                                animation.name.clone(),
                                Animation {
                                    frames,
                                    name: animation.name,
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
                        tilesets.insert(uid, tilset);
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
        Content {
            map: Map::new(&project),
            ldkt: project,
            tilesets,
            textures,
            sprites,
            tracks,
        }
    }
}