use std::collections::HashMap;

use crate::{
    assets::{AnimationsGroup, Assets},
    player::Tag,
    utils::*,
};
use asefile::AsepriteFile;
use macroquad::prelude::*;

pub struct Character<'a> {
    pub draw_pos: Vec2,
    pub actions: Vec<(ActionCondition, Action)>,
    pub animation: Option<&'a AnimationsGroup>,
    pub x: usize,
    pub y: usize,
    pub action_index: usize,
    pub animation_playing: bool,
    pub animation_index: usize,
    pub anim_time: f32,
    pub timer: f32,
    pub draw_over: bool,
    pub interact_message: Option<&'static str>,
    pub interacting: bool,
}
impl<'a> Character<'a> {
    pub fn get_action(&self) -> &(ActionCondition, Action) {
        if self.action_index >= self.actions.len() {
            return &NOOP_ACTION;
        }
        &self.actions[self.action_index]
    }
    pub fn draw(&self, assets: &Assets, ctx: &DrawCtx) {
        let time = (self.anim_time * 1000.0) as u32;
        if let Some(animation) = self.animation {
            draw_texture_ex(
                &animation.animations[self.animation_index].get_at_time(time),
                self.draw_pos.x * ctx.scale_factor
                    + (-ctx.camera_pos.x * ctx.scale_factor
                        + SCREEN_WIDTH * ctx.scale_factor / 2.0)
                        .floor(),
                self.draw_pos.y * ctx.scale_factor
                    + (-ctx.camera_pos.y * ctx.scale_factor
                        + SCREEN_HEIGHT * ctx.scale_factor / 2.0)
                        .floor(),
                WHITE,
                DrawTextureParams {
                    dest_size: Some(
                        animation.animations[0].get_at_time(0).size() * ctx.scale_factor,
                    ),
                    ..Default::default()
                },
            );
        }
    }
}

pub enum ActionCondition {
    AlwaysChange,
    NeverChange,
    PlayerInteract(&'static str, Vec2),
    PlayerHasTag(Tag),
    AnimationFinish,
    Dialogue(&'static str, &'static str),
    Time(f32),
}
pub enum Action {
    GiveTag(Tag),
    ChangeAnimation(usize),
    Teleport(usize, usize),
    TeleportPlayer(usize, usize),
    SetPlayingAnimation(bool),
    SetAnimationTime(f32),
    ShowScreen(usize),
    HideScreen,
    Noop,
}

pub const NOOP_ACTION: (ActionCondition, Action) = (ActionCondition::NeverChange, Action::Noop);

pub static BASE_CHARACTER: Character = Character {
    draw_pos: Vec2::ZERO,
    actions: Vec::new(),
    animation: None,
    x: 0,
    y: 0,
    action_index: 0,
    animation_index: 0,
    animation_playing: false,
    anim_time: 0.0,
    timer: 0.0,
    draw_over: false,
    interacting: false,
    interact_message: None,
};

pub fn raincoat_ferret<'a>((x, y): (usize, usize), assets: &'a Assets) -> Character<'a> {
    Character {
        draw_pos: vec2(x as f32, y as f32) * 16.0,
        actions: vec![
            (ActionCondition::PlayerHasTag(Tag::OpenedDoor), Action::Noop),
            (ActionCondition::Time(0.8), Action::Noop),
            (
                ActionCondition::Dialogue(
                    "Hello kind stranger! I have lost my way\nin the snowstorm. It is cold and dark.",
                    "Ferret in a raincoat",
                ),
                Action::Noop,
            ),
            (
                ActionCondition::Dialogue("Can I please come inside?", "Ferret in a raincoat"),
                Action::Noop,
            ),
            (ActionCondition::Time(0.5), Action::ShowScreen(1)),
            (ActionCondition::Time(1.0), Action::HideScreen),
            (
                ActionCondition::AlwaysChange,
                Action::TeleportPlayer(x + 1, y + 1),
            ),
            (ActionCondition::AlwaysChange, Action::Teleport(x, y + 1)),
            (
                ActionCondition::AlwaysChange,
                Action::GiveTag(Tag::ClosedDoor),
            ),
            (
                ActionCondition::AlwaysChange,
                Action::SetPlayingAnimation(true),
            ),
            (
                ActionCondition::AnimationFinish,
                Action::SetPlayingAnimation(false),
            ),
        ],
        animation: Some(&assets.raincoat_ferret),
        x,
        y,
        ..BASE_CHARACTER
    }
}
pub fn door<'a>((x, y): (usize, usize), assets: &'a Assets) -> Character<'a> {
    Character {
        draw_pos: vec2(x as f32, y as f32) * 16.0,
        actions: vec![
            (
                ActionCondition::PlayerInteract(
                    "E: open door",
                    vec2(x as f32, (y + 1) as f32) * 16.0,
                ),
                Action::SetPlayingAnimation(true),
            ),
            (
                ActionCondition::AnimationFinish,
                Action::SetPlayingAnimation(false),
            ),
            (
                ActionCondition::AlwaysChange,
                Action::GiveTag(Tag::OpenedDoor),
            ),
            (ActionCondition::AlwaysChange, Action::ShowScreen(0)),
            (
                ActionCondition::PlayerHasTag(Tag::ClosedDoor),
                Action::SetAnimationTime(0.0),
            ),
        ],
        animation: Some(&assets.door),
        x,
        y,
        ..BASE_CHARACTER
    }
}
pub fn fireplace<'a>((x, y): (usize, usize), assets: &'a Assets) -> Character<'a> {
    Character {
        draw_pos: vec2(x as f32, y as f32) * 16.0,
        actions: vec![
            (ActionCondition::PlayerHasTag(Tag::ClosedDoor), Action::Noop),
            (
                ActionCondition::PlayerInteract(
                    "E: light fireplace",
                    vec2(x as f32 + 0.5, (y + 2) as f32) * 16.0,
                ),
                Action::SetPlayingAnimation(true),
            ),
            (ActionCondition::AlwaysChange, Action::ChangeAnimation(1)),
        ],
        animation: Some(&assets.fireplace),
        x,
        y,
        draw_over: true,
        ..BASE_CHARACTER
    }
}

pub struct DrawCtx<'a> {
    pub screen_size: Vec2,
    pub camera_pos: Vec2,
    pub scale_factor: f32,
    pub assets: &'a Assets,
}

pub const DARK_BLUE: Color = Color::from_hex(0x143464);
pub const DIALOGUE_BORDER: Color = Color::from_hex(0xbb7547);
pub const DIALOGUE_BODY: Color = Color::from_hex(0x3b1725);

pub fn draw_dialogue(text: &str, name: &str, ctx: &DrawCtx) -> bool {
    let w = 200.0 * ctx.scale_factor;
    let h = 30.0 * ctx.scale_factor;
    let x = (ctx.screen_size.x - w) - 20.0 * ctx.scale_factor;
    let y = ctx.screen_size.y - h - 5.0 * ctx.scale_factor;
    draw_rectangle(x, y, w, h, DIALOGUE_BODY);
    draw_rectangle_lines(x, y, w, h, 2.0 * ctx.scale_factor, DIALOGUE_BORDER);
    let nameplate_height = 10.0 * ctx.scale_factor;
    draw_rectangle(
        x,
        y - nameplate_height + 1.0 * ctx.scale_factor,
        80.0 * ctx.scale_factor,
        nameplate_height,
        DIALOGUE_BODY,
    );
    draw_rectangle_lines(
        x,
        y - nameplate_height + 1.0 * ctx.scale_factor,
        80.0 * ctx.scale_factor,
        nameplate_height,
        2.0 * ctx.scale_factor,
        DIALOGUE_BORDER,
    );
    draw_text_ex(
        name,
        x + 1.0 * ctx.scale_factor,
        y - 2.0 * ctx.scale_factor,
        TextParams {
            font: Some(&ctx.assets.font),
            font_size: (8.0 * ctx.scale_factor) as u16,
            ..Default::default()
        },
    );
    draw_multiline_text_ex(
        text,
        x + 5.0 * ctx.scale_factor,
        y + 12.0 * ctx.scale_factor,
        None,
        TextParams {
            font: Some(&ctx.assets.font),
            font_size: (12.0 * ctx.scale_factor) as u16,
            ..Default::default()
        },
    );
    is_key_pressed(KeyCode::E)
}

pub fn draw_tooltip(text: &str, ctx: &DrawCtx) -> bool {
    let w = 150.0 * ctx.scale_factor;
    let h = 20.0 * ctx.scale_factor;
    let x = (ctx.screen_size.x - w) / 2.0;
    let y = ctx.screen_size.y - h - 5.0 * ctx.scale_factor;
    draw_rectangle(x, y, w, h, DARK_BLUE);
    draw_rectangle_lines(x, y, w, h, 2.0 * ctx.scale_factor, WHITE);
    draw_text_ex(
        text,
        x + 5.0 * ctx.scale_factor,
        y + 12.0 * ctx.scale_factor,
        TextParams {
            font: Some(&ctx.assets.font),
            font_size: (12.0 * ctx.scale_factor) as u16,
            ..Default::default()
        },
    );
    is_key_pressed(KeyCode::E)
}
