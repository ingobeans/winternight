use macroquad::prelude::*;

use crate::{assets::Assets, utils::get_input_axis};

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    fn name(&self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
    fn from_vec2(vec: Vec2, last: Vec2) -> Self {
        if !(vec.x != 0.0 && vec.y != 0.0) {
            match (vec.x, vec.y) {
                (0.0, -1.0) => Direction::Up,
                (0.0, 1.0) => Direction::Down,
                (1.0, 0.0) => Direction::Right,
                (-1.0, 0.0) => Direction::Left,
                _ => panic!(),
            }
        } else {
            let x_dir = Self::from_vec2(vec2(vec.x, 0.0), Vec2::ZERO);
            let y_dir = Self::from_vec2(vec2(0.0, vec.y), Vec2::ZERO);
            if last.x != 0.0 { y_dir } else { x_dir }
        }
    }
    fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => vec2(0.0, -1.0),
            Direction::Down => vec2(0.0, 1.0),
            Direction::Right => vec2(1.0, 0.0),
            Direction::Left => vec2(-1.0, 0.0),
            _ => panic!(),
        }
    }
}
pub enum PlayerState {
    Moving,
    Idle,
}

const MOVE_TIME: f32 = 0.25;

pub struct Player {
    pub draw_pos: Vec2,
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
    pub time: f32,
    pub state: PlayerState,
}
impl Player {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            draw_pos: vec2(x as f32, y as f32) * 16.0,
            x,
            y,
            direction: Direction::Left,
            time: 0.0,
            state: PlayerState::Idle,
        }
    }
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        match self.state {
            PlayerState::Idle => {
                let axis = get_input_axis();
                if axis != Vec2::ZERO {
                    self.direction = Direction::from_vec2(axis, self.direction.to_vec2());
                    self.state = PlayerState::Moving;
                    let dir = self.direction.to_vec2();
                    self.x = self.x.saturating_add_signed(dir.x as isize);
                    self.y = self.y.saturating_add_signed(dir.y as isize);
                }
            }
            PlayerState::Moving => {
                let target = vec2(self.x as f32, self.y as f32) * 16.0;
                if self.draw_pos.distance(target) <= delta_time * (16.0 / MOVE_TIME) {
                    self.draw_pos = target;
                    self.state = PlayerState::Idle;
                } else {
                    self.draw_pos = self
                        .draw_pos
                        .move_towards(target, delta_time * (16.0 / MOVE_TIME));
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
            self.draw_pos.x.floor() * scale_factor,
            self.draw_pos.y.floor() * scale_factor,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(16.0, 16.0) * scale_factor),
                ..Default::default()
            },
        );
    }
}
