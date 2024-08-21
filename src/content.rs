use engine::graphics::texture::Texture;

pub struct Content {
    atlas: Texture,
}

impl Content {
    pub fn new() -> Self {
        // TODO: Async?
        Content {
            atlas: Texture::from_path("src/atlas.png"),
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
