use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::assets::Assets;
use crate::characters::*;
use crate::player::{Direction, MOVE_TIME, Player};
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
                fireplace(assets.map.special.find_tile(3), assets),
                door(assets.map.special.find_tile(2), assets),
                raincoat_ferret(assets.map.special.find_tile(1), assets),
                //test_character(assets.map.special.find_tile(4), assets),
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
        let mut ctx = DrawCtx {
            screen_size: vec2(screen_width, screen_height),
            camera_pos: self.player.draw_pos.floor(),
            scale_factor,
            assets: &self.assets,
        };
        let interacting_with_any = any_interacting(&self.characters);
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
            if interacting_with_any.is_none() {
                self.player
                    .update(delta_time, self.assets, &mut self.characters);
            }
            ctx.camera_pos = self.player.draw_pos.floor();
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
                -self.player.draw_pos.x.floor() * scale_factor
                    + SCREEN_WIDTH * scale_factor / 2.0
                    + SCREEN_WIDTH * scale_factor / 2.0
                    - 118.0 * scale_factor
                    - 12.0 * scale_factor,
                -self.player.draw_pos.y.floor() * scale_factor
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
                -self.player.draw_pos.x.floor() * scale_factor
                    + SCREEN_WIDTH * scale_factor / 2.0
                    + SCREEN_WIDTH * scale_factor / 2.0
                    - 118.0 * scale_factor
                    - 12.0 * scale_factor,
                -self.player.draw_pos.y.floor() * scale_factor
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
                (-self.player.draw_pos.x.floor() * scale_factor
                    + SCREEN_WIDTH * scale_factor / 2.0)
                    .floor(),
                (-self.player.draw_pos.y.floor() * scale_factor
                    + SCREEN_HEIGHT * scale_factor / 2.0)
                    .floor(),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(map.texture.size() * scale_factor),
                    ..Default::default()
                },
            );
            self.player.draw(self.assets, scale_factor);
            for character in self.characters.iter().filter(|f| !f.draw_over).rev() {
                character.draw(self.assets, &ctx);
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
                (-self.player.draw_pos.x.floor() * scale_factor
                    + SCREEN_WIDTH * scale_factor / 2.0)
                    .floor(),
                (-self.player.draw_pos.y.floor() * scale_factor
                    + SCREEN_HEIGHT * scale_factor / 2.0)
                    .floor(),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(map.texture.size() * scale_factor),
                    ..Default::default()
                },
            );
            for character in self.characters.iter().filter(|f| f.draw_over).rev() {
                character.draw(self.assets, &ctx);
            }
        }
        for character in self.characters.iter_mut() {
            character.timer += delta_time;
            let mut reached_destination = false;
            if let Some((x, y)) = &character.moving_to {
                let target = vec2(character.x as f32, character.y as f32) * 16.0;

                if character.draw_pos.distance(target) <= delta_time * (16.0 / MOVE_TIME) {
                    character.draw_pos = target;
                    let path = pathfind(
                        self.assets,
                        (character.x, character.y),
                        (*x, *y),
                        (self.player.x, self.player.y),
                    );
                    if let Some(path) = path.and_then(|f| f.0.get(1).cloned()) {
                        (character.x, character.y) = path;
                    }
                } else {
                    let delta = target - character.draw_pos;
                    character.direction = Direction::from_vec2(delta.normalize(), Vec2::ZERO);
                    character.animation_index =
                        character.animation.unwrap().tag_names[character.direction.name()];
                    character.draw_pos = character
                        .draw_pos
                        .move_towards(target, delta_time * (16.0 / MOVE_TIME));
                }

                reached_destination = vec2((x * 16) as f32, (y * 16) as f32) == character.draw_pos;
            }
            if character.animation_playing {
                character.anim_time += delta_time;
            }
            if character.interacting
                && let Some(text) = character.interact_message
            {
                if draw_dialogue(text, character.name, &ctx) {
                    character.interacting = false;
                }
            }
            let mut set_time = None;
            let action = character.get_action();
            if match &action.0 {
                ActionCondition::ReachedDestination => reached_destination,
                ActionCondition::PlayerHasTag(tag) => self.player.tags.contains(tag),
                ActionCondition::PlayerInteract(text, pos) => {
                    let dist = self.player.draw_pos.distance_squared(*pos);
                    if dist <= 350.0 {
                        draw_tooltip(&text, &ctx)
                    } else {
                        false
                    }
                }
                ActionCondition::AlwaysChange => true,
                ActionCondition::NeverChange => false,
                ActionCondition::AnimationFinish => {
                    if let Some(animation) = character.animation {
                        let anim_length =
                            animation.animations[character.animation_index].total_length as f32;
                        let result = character.animation_playing
                            && character.anim_time * 1000.0 >= anim_length;
                        if result {
                            set_time = Some((anim_length - 1.0) / 1000.0);
                        }
                        result
                    } else {
                        false
                    }
                }
                ActionCondition::Dialogue(text) => draw_dialogue(text, character.name, &ctx),
                ActionCondition::Time(time) => character.timer >= *time,
            } {
                match &action.1 {
                    Action::Noop => {}
                    Action::MoveTo(pos) => character.moving_to = Some(*pos),
                    Action::ChangeAnimation(index) => {
                        character.animation_index = *index;
                        character.anim_time = 0.0;
                    }
                    Action::SetPlayingAnimation(value) => character.animation_playing = *value,
                    Action::ShowScreen(index) => self.screen = Some(*index),
                    Action::HideScreen => self.screen = None,
                    Action::GiveTag(tag) => self.player.tags.push(*tag),
                    Action::SetAnimationTime(time) => set_time = Some(*time),
                    Action::SetInteractMessage(msg) => character.interact_message = *msg,
                    Action::SetCollision(value) => character.has_collision = *value,
                    Action::Teleport(x, y) => {
                        let x = *x;
                        let y = *y;
                        character.x = x;
                        character.y = y;
                        character.draw_pos =
                            vec2((character.x * 16) as f32, (character.y * 16) as f32);
                    }
                    Action::TeleportPlayer(x, y) => {
                        let x = *x;
                        let y = *y;
                        self.player.x = x;
                        self.player.y = y;
                        self.player.draw_pos =
                            vec2((self.player.x * 16) as f32, (self.player.y * 16) as f32);
                    }
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
