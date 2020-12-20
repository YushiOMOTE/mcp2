use bevy::{prelude::*, sprite::Rect};

pub struct AtlasBuilder {
    texture: Handle<Texture>,
    dimensions: Vec2,
    offset: Vec2,
    padding: Vec2,
    size: Vec2,
    scale: Vec2,
}

impl AtlasBuilder {
    pub fn load(asset_server: &AssetServer, size: Vec2, dimensions: Vec2, path: &str) -> Self {
        let texture = asset_server.load(path);
        Self {
            texture,
            dimensions,
            offset: Vec2::zero(),
            padding: Vec2::zero(),
            size,
            scale: Vec2::one(),
        }
    }

    pub fn offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }

    pub fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn build(mut self, atlases: &mut Assets<TextureAtlas>) -> Handle<TextureAtlas> {
        self.dimensions *= self.scale;
        self.offset *= self.scale;
        self.padding *= self.scale;
        self.size *= self.scale;

        let mut atlas = TextureAtlas::new_empty(self.texture, self.dimensions);

        let mut x = 0.0;
        let mut y = 0.0;

        loop {
            let min_x = x + self.padding.x + self.offset.x;
            let min_y = y + self.padding.y + self.offset.y;
            let max_x = min_x + self.size.x - self.padding.x * 2.0;
            let max_y = min_y + self.size.y - self.padding.y * 2.0;

            if max_x > self.dimensions.x {
                x = 0.0;
                y += self.size.y;
                continue;
            }

            if max_y > self.dimensions.y {
                break;
            }

            atlas.add_texture(Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            });

            x += self.size.x;
        }

        atlases.add(atlas)
    }
}
