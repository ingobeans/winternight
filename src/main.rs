use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::assets::Assets;
use crate::player::Player;
use crate::utils::*;

mod assets;
mod player;
mod utils;

struct Game<'a> {
    assets: &'a Assets,
    player: Player,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        let (x, y) = assets.map.special.find_tile(0);
        Self {
            assets,
            player: Player::new(x, y),
        }
    }
    fn update(&mut self) {
        set_default_camera();
        clear_background(BLACK);
        let (screen_width, screen_height) = screen_size();
        let scale_factor = (screen_width / SCREEN_WIDTH).min(screen_height / SCREEN_HEIGHT);
        let delta_time = get_frame_time();
        self.player.update(delta_time, self.assets);
        let map = self
            .assets
            .map
            .background_camera
            .render_target
            .as_ref()
            .unwrap();

        draw_texture_ex(
            &map.texture,
            -self.player.draw_pos.x * scale_factor + SCREEN_WIDTH * scale_factor / 2.0,
            -self.player.draw_pos.y * scale_factor + SCREEN_HEIGHT * scale_factor / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(map.texture.size() * scale_factor),
                ..Default::default()
            },
        );
        self.player.draw(self.assets, scale_factor);
        let map = self
            .assets
            .map
            .foreground_camera
            .render_target
            .as_ref()
            .unwrap();
        draw_texture_ex(
            &map.texture,
            -self.player.draw_pos.x * scale_factor + SCREEN_WIDTH * scale_factor / 2.0,
            -self.player.draw_pos.y * scale_factor + SCREEN_HEIGHT * scale_factor / 2.0,
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
