use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::assets::Assets;
use crate::utils::*;

mod assets;
mod utils;

struct Game<'a> {
    assets: &'a Assets,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self { assets }
    }
    fn update(&mut self) {
        set_default_camera();
        clear_background(BLACK);
        let (screen_width, screen_height) = screen_size();
        let scale_factor = (screen_width / SCREEN_WIDTH).min(screen_height / SCREEN_HEIGHT);
        let map = self.assets.map.camera.render_target.as_ref().unwrap();
        draw_texture_ex(
            &map.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(map.texture.size() * scale_factor),
                ..Default::default()
            },
        );
    }
}

#[macroquad::main("winternight")]
async fn main() {
    let assets = Assets::load();
    let mut game = Game::new(&assets);
    loop {
        game.update();
        next_frame().await
    }
}
