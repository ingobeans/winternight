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
    time: f32,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        let (x, y) = assets.map.special.find_tile(0);
        Self {
            assets,
            player: Player::new(x, y),
            time: 0.0,
        }
    }
    fn update(&mut self) {
        set_default_camera();
        clear_background(BLACK);
        let (screen_width, screen_height) = screen_size();
        let scale_factor = (screen_width / SCREEN_WIDTH).min(screen_height / SCREEN_HEIGHT);
        let delta_time = get_frame_time();
        self.time += delta_time;
        self.player.update(delta_time, self.assets);
        let map = self
            .assets
            .map
            .background_camera
            .render_target
            .as_ref()
            .unwrap();

        // draw vision cones.
        // i did this by hand and it uses a lot of magic numbers, mb
        draw_texture_ex(
            &self.assets.vision_cones,
            -self.player.draw_pos.x * scale_factor
                + SCREEN_WIDTH * scale_factor / 2.0
                + SCREEN_WIDTH * scale_factor / 2.0
                - 118.0 * scale_factor
                - 12.0 * scale_factor,
            -self.player.draw_pos.y * scale_factor
                + SCREEN_HEIGHT * scale_factor / 2.0
                + SCREEN_HEIGHT * scale_factor / 2.0
                - 12.0 * scale_factor,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.assets.vision_cones.size() * scale_factor * 1.15),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &self
                .assets
                .snow_blowing
                .get_at_time((self.time * 1000.0) as u32),
            -self.player.draw_pos.x * scale_factor
                + SCREEN_WIDTH * scale_factor / 2.0
                + SCREEN_WIDTH * scale_factor / 2.0
                - 118.0 * scale_factor
                - 12.0 * scale_factor,
            -self.player.draw_pos.y * scale_factor
                + SCREEN_HEIGHT * scale_factor / 2.0
                + SCREEN_HEIGHT * scale_factor / 2.0
                - 12.0 * scale_factor,
            WHITE.with_alpha(0.5),
            DrawTextureParams {
                dest_size: Some(self.assets.vision_cones.size() * scale_factor * 1.15),
                ..Default::default()
            },
        );
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
