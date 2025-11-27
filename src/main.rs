use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::assets::Assets;
use crate::characters::*;
use crate::player::Player;
use crate::utils::*;

mod assets;
mod characters;
mod player;
mod utils;

struct Game<'a> {
    assets: &'a Assets,
    player: Player,
    time: f32,
    characters: Vec<Character<'a>>,
    screen: Option<usize>,
}
impl<'a> Game<'a> {
    fn new(assets: &'a Assets) -> Self {
        Self {
            assets,
            player: Player::new(assets.map.special.find_tile(0)),
            time: 0.0,
            characters: vec![
                door(assets.map.special.find_tile(2), assets),
                raincoat_ferret(assets.map.special.find_tile(1), assets),
            ],
            screen: None,
        }
    }
    fn update(&mut self) {
        set_default_camera();
        clear_background(BLACK);
        let (screen_width, screen_height) = screen_size();
        let scale_factor = (screen_width / SCREEN_WIDTH).min(screen_height / SCREEN_HEIGHT);
        let delta_time = get_frame_time();
        self.time += delta_time;
        let ctx = DrawCtx {
            screen_size: vec2(screen_width, screen_height),
            camera_pos: self.player.draw_pos,
            scale_factor,
            assets: &self.assets,
        };
        if let Some(screen) = &self.screen {
            let screen = &self.assets.screens[*screen];
            let size = screen.get_at_time(0).size() * scale_factor * 4.0;
            draw_texture_ex(
                &screen.get_at_time((self.time * 1000.0) as u32),
                (screen_width - size.x) / 2.0,
                (screen_height - size.y) / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(size),
                    ..Default::default()
                },
            );
        } else {
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
                (-self.player.draw_pos.x * scale_factor + SCREEN_WIDTH * scale_factor / 2.0)
                    .floor(),
                (-self.player.draw_pos.y * scale_factor + SCREEN_HEIGHT * scale_factor / 2.0)
                    .floor(),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(map.texture.size() * scale_factor),
                    ..Default::default()
                },
            );
            self.player.draw(self.assets, scale_factor);
            for character in self.characters.iter().rev() {
                let time = (character.anim_time * 1000.0) as u32;
                draw_texture_ex(
                    &character.animation.animations[character.animation_index].get_at_time(time),
                    character.draw_pos.x * scale_factor
                        + (-self.player.draw_pos.x * scale_factor
                            + SCREEN_WIDTH * scale_factor / 2.0)
                            .floor(),
                    character.draw_pos.y * scale_factor
                        + (-self.player.draw_pos.y * scale_factor
                            + SCREEN_HEIGHT * scale_factor / 2.0)
                            .floor(),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(
                            character.animation.animations[0].get_at_time(0).size() * scale_factor,
                        ),
                        ..Default::default()
                    },
                );
            }
            let map = self
                .assets
                .map
                .foreground_camera
                .render_target
                .as_ref()
                .unwrap();
            draw_texture_ex(
                &map.texture,
                (-self.player.draw_pos.x * scale_factor + SCREEN_WIDTH * scale_factor / 2.0)
                    .floor(),
                (-self.player.draw_pos.y * scale_factor + SCREEN_HEIGHT * scale_factor / 2.0)
                    .floor(),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(map.texture.size() * scale_factor),
                    ..Default::default()
                },
            );
        }
        for character in self.characters.iter_mut() {
            character.timer += delta_time;
            if character.animation_playing {
                character.anim_time += delta_time;
            }
            let mut set_time = None;
            let action = character.get_action();
            if match &action.0 {
                ActionCondition::PlayerHasTag(tag) => self.player.tags.contains(tag),
                ActionCondition::PlayerInteract(text, pos) => {
                    let dist = self.player.draw_pos.distance_squared(*pos);
                    if dist <= 256.0 {
                        draw_tooltip(&text, &ctx)
                    } else {
                        false
                    }
                }
                ActionCondition::AlwaysChange => true,
                ActionCondition::NeverChange => false,
                ActionCondition::AnimationFinish => {
                    let anim_length = character.animation.animations[character.animation_index]
                        .total_length as f32;
                    let result =
                        character.animation_playing && character.anim_time * 1000.0 >= anim_length;
                    if result {
                        set_time = Some((anim_length - 1.0) / 1000.0);
                    }
                    result
                }
                ActionCondition::Dialogue(text, name) => draw_dialogue(text, name, &ctx),
                ActionCondition::Time(time) => character.timer >= *time,
            } {
                match &action.1 {
                    Action::ChangeAnimation(index) => character.animation_index = *index,
                    Action::Noop => {}
                    Action::SetPlayingAnimation(value) => character.animation_playing = *value,
                    Action::ShowScreen(index) => self.screen = Some(*index),
                    Action::HideScreen => self.screen = None,
                    Action::GiveTag(tag) => self.player.tags.push(*tag),
                    _ => todo!(),
                }
                character.timer = 0.0;
                character.action_index += 1;
                if let Some(time) = set_time {
                    character.anim_time = time;
                }
            }
        }
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
