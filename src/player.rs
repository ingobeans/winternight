use macroquad::prelude::*;

use crate::{
    assets::Assets,
    characters::{Character, any_interacting},
    utils::*,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Tag {
    OpenedDoor,
    ClosedDoor,
    LightFire,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    pub fn name(&self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
    pub fn from_vec2(vec: Vec2, last: Vec2) -> Self {
        if !(vec.x != 0.0 && vec.y != 0.0) {
            if vec.x < 0.0 {
                Direction::Left
            } else if vec.x > 0.0 {
                Direction::Right
            } else if vec.y < 0.0 {
                Direction::Up
            } else if vec.y > 0.0 {
                Direction::Down
            } else {
                Direction::Left
            }
        } else {
            let x_dir = Self::from_vec2(vec2(vec.x, 0.0), Vec2::ZERO);
            let y_dir = Self::from_vec2(vec2(0.0, vec.y), Vec2::ZERO);
            if last.x != 0.0 { y_dir } else { x_dir }
        }
    }
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => vec2(0.0, -1.0),
            Direction::Down => vec2(0.0, 1.0),
            Direction::Right => vec2(1.0, 0.0),
            Direction::Left => vec2(-1.0, 0.0),
        }
    }
}
pub enum PlayerState {
    Moving,
    Idle,
}

pub const MOVE_TIME: f32 = 0.25;

pub struct Player {
    pub tags: Vec<Tag>,
    pub draw_pos: Vec2,
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub time: f32,
    pub state: PlayerState,
}
impl Player {
    pub fn new((x, y): (usize, usize)) -> Self {
        Self {
            tags: Vec::new(),
            draw_pos: vec2(x as f32, y as f32) * 16.0,
            x,
            y,
            direction: Direction::Left,
            time: 0.0,
            state: PlayerState::Idle,
        }
    }
    pub fn update(&mut self, delta_time: f32, assets: &Assets, characters: &mut Vec<Character>) {
        self.time += delta_time;
        let interacting_with_any = any_interacting(&characters).is_some();
        match self.state {
            PlayerState::Idle => {
                let axis = get_input_axis();
                if axis != Vec2::ZERO {
                    self.direction = Direction::from_vec2(axis, self.direction.to_vec2());
                    let dir = self.direction.to_vec2();
                    let new_x = self.x.saturating_add_signed(dir.x as isize);
                    let new_y = self.y.saturating_add_signed(dir.y as isize);

                    if assets.map.walls.0[new_x + new_y * assets.map.walls.1] == 0 {
                        if let Some(character) = characters
                            .iter_mut()
                            .find(|f| f.has_collision && f.x == new_x && f.y == new_y)
                        {
                            if !interacting_with_any && character.interact_message.is_some() {
                                character.interacting = true;
                                let dir = Direction::from_vec2(
                                    (self.draw_pos - character.draw_pos).normalize(),
                                    Vec2::ZERO,
                                )
                                .name();
                                character.animation_index =
                                    character.animation.unwrap().tag_names[dir];
                            }
                        } else {
                            (self.x, self.y) = (new_x, new_y);
                            self.state = PlayerState::Moving;
                        }
                    }
                }
            }
            PlayerState::Moving => {
                let target = vec2(self.x as f32, self.y as f32) * 16.0;
                #[cfg(debug_assertions)]
                let move_time = if is_key_down(KeyCode::LeftShift) {
                    0.1
                } else {
                    MOVE_TIME
                };
                #[cfg(not(debug_assertions))]
                let move_time = MOVE_TIME;
                if self.draw_pos.distance(target) <= delta_time * (16.0 / move_time) {
                    self.draw_pos = target;
                    self.state = PlayerState::Idle;
                } else {
                    self.draw_pos = self
                        .draw_pos
                        .move_towards(target, delta_time * (16.0 / move_time));
                }
            }
        }
    }
    pub fn draw(&self, assets: &Assets, scale_factor: f32) {
        let anim_frame = if let PlayerState::Moving = self.state {
            (self.time * 1000.0) as u32
        } else {
            0
        };
        draw_texture_ex(
            &assets
                .player
                .get_by_name(self.direction.name())
                .get_at_time(anim_frame),
            SCREEN_WIDTH * scale_factor / 2.0,
            SCREEN_HEIGHT * scale_factor / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(16.0, 16.0) * scale_factor),
                ..Default::default()
            },
        );
    }
}
