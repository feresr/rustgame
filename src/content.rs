use std::{collections::HashMap, fs};

use engine::{
    audio::AudioTrack,
    graphics::{
        common::RectF,
        texture::{SubTexture, Texture},
    },
};

use crate::{
    aseprite::{self, Aseprite},
    components::sprite::{Animation, Frame},
};

pub struct Content {
    atlas: Texture,
    pub textures: HashMap<String, Texture>,
    // animation sets
    pub sprites: HashMap<String, HashMap<String, Animation>>,
    pub tracks: HashMap<&'static str, AudioTrack>,
}

impl Content {
    pub fn new() -> Self {
        // TODO: Async?
        let mut textures = HashMap::new();
        let mut sprites = HashMap::new();

        let assets = fs::read_dir("src/assets/").unwrap();
        for asset in assets {
            let path = asset.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "bin" {
                    if let Some(path_str) = path.to_str() {
                        // todo: Repalce .bin with .png
                        let png_str = path_str.replace(".bin", ".png");
                        let texture = Texture::from_path(&png_str);
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
                                        texture,
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
            }
        }

        // TODO: Load all audio in folder
        let mut tracks = HashMap::new();
        let audio = AudioTrack::new("src/assets/song.ogg").unwrap();
        tracks.insert("music-1", audio);
        let audio = AudioTrack::new("src/assets/jump.ogg").unwrap();
        tracks.insert("jump", audio);
        Content {
            atlas: Texture::from_path("src/atlas.png"),
            textures,
            sprites,
            tracks,
        }
    }

    pub fn altas(&self) -> &Texture {
        return &self.atlas;
    }
}

static mut CONTENT: Option<Content> = None;

pub fn content() -> &'static Content {
    unsafe {
        if CONTENT.is_none() {
            CONTENT = Some(Content::new());
        }
        CONTENT.as_ref().unwrap()
    }
}
