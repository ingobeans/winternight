use std::collections::HashMap;

use asefile::AsepriteFile;
use image::EncodableLayout;
use macroquad::prelude::*;

use crate::utils::create_camera;

// hello!

pub struct Assets {
    pub tileset: Spritesheet,
    pub map: Map,
    pub player: AnimationsGroup,
}
impl Assets {
    pub fn load() -> Self {
        let tileset = Spritesheet::new(
            load_ase_texture(include_bytes!("../assets/tileset.ase"), None),
            16.0,
        );
        Self {
            map: Map::new(include_str!("../assets/map.tmx"), &tileset),
            tileset,
            player: AnimationsGroup::from_file(include_bytes!("../assets/player.ase")),
        }
    }
}

pub struct Map {
    pub background_camera: Camera2D,
    pub foreground_camera: Camera2D,
    pub floor: TileMap,
    pub walls: TileMap,
    pub detail: TileMap,
}
impl Map {
    pub fn new(data: &str, tileset: &Spritesheet) -> Self {
        let floor = parse_tilemap_layer(data, "floor");
        let w = floor.1 as f32 * 16.0;
        let h = (floor.0.len() / floor.1) as f32 * 16.0;
        let mut background_camera = create_camera(w, h);
        background_camera.target = vec2(w / 2.0, h / 2.0);
        let mut foreground_camera = create_camera(w, h);
        foreground_camera.target = vec2(w / 2.0, h / 2.0);
        let new = Self {
            background_camera,
            foreground_camera,
            floor,
            walls: parse_tilemap_layer(data, "walls"),
            detail: parse_tilemap_layer(data, "detail"),
        };
        set_camera(&new.background_camera);
        new.floor.draw(tileset);
        set_camera(&new.foreground_camera);
        new.walls.draw(tileset);
        new.detail.draw(tileset);
        new
    }
}

pub struct TileMap(Vec<u8>, usize);
impl TileMap {
    fn draw(&self, tileset: &Spritesheet) {
        let spritesheet_width = (tileset.texture.width() / tileset.sprite_size) as u8;
        for (index, tile) in self.0.iter().enumerate() {
            if *tile == 0 {
                continue;
            }
            let tile = tile - 1;
            let x = index % self.1;
            let y = index / self.1;

            tileset.draw_tile(
                (x * 16) as f32,
                (y * 16) as f32,
                (tile % spritesheet_width) as f32,
                (tile / spritesheet_width) as f32,
                None,
            );
        }
    }
}

pub fn parse_tilemap_layer(xml: &str, layer_name: &str) -> TileMap {
    let pattern = format!("name=\"{layer_name}\" ");

    let width = xml
        .split_once("width=\"")
        .unwrap()
        .1
        .split_once("\"")
        .unwrap()
        .0
        .parse()
        .unwrap();
    let xml = xml
        .split_once(&pattern)
        .unwrap()
        .1
        .split_once("<data encoding=\"csv\">")
        .unwrap()
        .1
        .split_once("</data>")
        .unwrap()
        .0;
    let mut split = xml.split(',');
    let mut data = vec![0];
    while let Some(tile) = split.next() {
        data.push(tile.trim().parse().unwrap());
    }
    TileMap(data, width)
}

pub struct Spritesheet {
    pub texture: Texture2D,
    pub sprite_size: f32,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: f32) -> Self {
        Self {
            texture,
            sprite_size,
        }
    }
    /// Same as `draw_tile`, except centered
    pub fn draw_sprite(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        self.draw_tile(
            screen_x - self.sprite_size / 2.0,
            screen_y - self.sprite_size / 2.0,
            tile_x,
            tile_y,
            params,
        );
    }
    /// Draws a single tile from the spritesheet
    pub fn draw_tile(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        let mut p = params.cloned().unwrap_or(DrawTextureParams::default());
        p.dest_size = p
            .dest_size
            .or(Some(Vec2::new(self.sprite_size, self.sprite_size)));
        p.source = p.source.or(Some(Rect {
            x: tile_x * self.sprite_size,
            y: tile_y * self.sprite_size,
            w: self.sprite_size,
            h: self.sprite_size,
        }));
        draw_texture_ex(&self.texture, screen_x, screen_y, WHITE, p);
    }
}
pub struct AnimationsGroup {
    #[expect(dead_code)]
    pub file: AsepriteFile,
    pub animations: Vec<Animation>,
    pub tag_names: HashMap<String, usize>,
}
impl AnimationsGroup {
    pub fn get_by_name(&self, name: &str) -> &Animation {
        &self.animations[*self.tag_names.get(name).unwrap()]
    }
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        let mut tag_frames = Vec::new();
        let mut offset = 0;

        let mut tag_names = HashMap::new();

        for i in 0..ase.num_tags() {
            let tag = ase.get_tag(i).unwrap();
            tag_names.insert(tag.name().to_string(), i as usize);
            let (start, end) = (tag.from_frame() as usize, tag.to_frame() as usize);
            let mut total_length = 0;
            let included_frames: Vec<(Texture2D, u32)> = frames
                .extract_if((start - offset)..(end - offset + 1), |_| true)
                .collect();
            for f in included_frames.iter() {
                total_length += f.1;
            }
            offset += end.abs_diff(start) + 1;
            tag_frames.push(Animation {
                frames: included_frames,
                total_length,
            });
        }
        Self {
            file: ase,
            animations: tag_frames,
            tag_names,
        }
    }
}
pub struct Animation {
    frames: Vec<(Texture2D, u32)>,
    pub total_length: u32,
}
impl Animation {
    pub fn from_file(bytes: &[u8]) -> Self {
        let ase = AsepriteFile::read(bytes).unwrap();
        let mut frames = Vec::new();
        let mut total_length = 0;
        for index in 0..ase.num_frames() {
            let frame = ase.frame(index);
            let img = frame.image();
            let new = Image {
                width: img.width() as u16,
                height: img.height() as u16,
                bytes: img.as_bytes().to_vec(),
            };
            let duration = frame.duration();
            total_length += duration;
            let texture = Texture2D::from_image(&new);
            texture.set_filter(FilterMode::Nearest);
            frames.push((texture, duration));
        }
        Self {
            frames,
            total_length,
        }
    }
    pub fn get_at_time(&self, mut time: u32) -> &Texture2D {
        time %= self.total_length;
        for (texture, length) in self.frames.iter() {
            if time >= *length {
                time -= length;
            } else {
                return texture;
            }
        }
        panic!()
    }
}

fn load_ase_texture(bytes: &[u8], layer: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(0).image()
    } else {
        img.frame(0).image()
    };
    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}
