use std::iter::Map;

use crate::{
    assets::{AnimationsGroup, Assets},
    player::{Direction, Tag},
    utils::*,
};
use macroquad::prelude::*;

pub fn any_interacting(characters: &[Character]) -> Option<usize> {
    characters.iter().position(|f| {
        f.interacting || {
            let action = f.get_action();
            match &action.0 {
                ActionCondition::Dialogue(_) => true,
                _ => false,
            }
        }
    })
}
type SuccessorIterator =
    Map<std::vec::IntoIter<(usize, usize)>, fn((usize, usize)) -> ((usize, usize), usize)>;

pub fn pathfind(
    assets: &Assets,
    from: (usize, usize),
    to: (usize, usize),
    player_pos: (usize, usize),
) -> Option<(Vec<(usize, usize)>, usize)> {
    pathfinding::prelude::astar(
        &from,
        |p| generate_successors(assets, *p, player_pos),
        |&(x, y)| {
            ((to.0 as f32 - x as f32).powi(2) + (to.1 as f32 - y as f32).powi(2)).sqrt() as usize
        },
        |&p| p == to,
    )
}

fn generate_successors(
    assets: &Assets,
    pos: (usize, usize),
    player_pos: (usize, usize),
) -> SuccessorIterator {
    let (x, y) = pos;
    let mut candidates = vec![(x + 1, y), (x, y + 1)];
    if x > 0 {
        candidates.push((x - 1, y));
    }
    if y > 0 {
        candidates.push((x, y - 1));
    }
    candidates.retain(|(cx, cy)| {
        (*cx, *cy) != player_pos && assets.map.walls.0[cx + cy * assets.map.walls.1] == 0
    });
    fn map_function(p: (usize, usize)) -> ((usize, usize), usize) {
        (p, 1)
    }
    let mapped: SuccessorIterator = candidates.into_iter().map(map_function);
    mapped
}

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
    pub name: &'static str,
    pub moving_to: Option<(usize, usize)>,
    pub direction: Direction,
    pub has_collision: bool,
    pub draw_offset: Vec2,
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
                (self.draw_pos.x + self.draw_offset.x) * ctx.scale_factor
                    + (-ctx.camera_pos.x * ctx.scale_factor
                        + SCREEN_WIDTH * ctx.scale_factor / 2.0)
                        .floor(),
                (self.draw_pos.y + self.draw_offset.y) * ctx.scale_factor
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
    ReachedDestination,
    PlayerHasTag(Tag),
    AnimationFinish,
    Dialogue(&'static str),
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
    SetInteractMessage(Option<&'static str>),
    MoveTo((usize, usize)),
    HideScreen,
    SetCollision(bool),
    SetName(&'static str),
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
    moving_to: None,
    name: "",
    direction: Direction::Left,
    has_collision: true,
    draw_offset: Vec2::ZERO,
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
                ),
                Action::Noop,
            ),
            (
                ActionCondition::Dialogue("Can I please come inside?"),
                Action::Noop,
            ),
            (ActionCondition::Time(0.5), Action::ShowScreen(1)),
            (ActionCondition::Time(1.0), Action::HideScreen),
            (
                ActionCondition::AlwaysChange,
                Action::TeleportPlayer(x + 1, y + 1),
            ),
            (ActionCondition::AlwaysChange, Action::Teleport(x, y + 1)),
            (ActionCondition::AlwaysChange, Action::SetName("Ferret")),
            (
                ActionCondition::AlwaysChange,
                Action::GiveTag(Tag::ClosedDoor),
            ),
            (
                ActionCondition::AlwaysChange,
                Action::SetPlayingAnimation(true),
            ),
            (ActionCondition::AnimationFinish, Action::ChangeAnimation(3)),
            (
                ActionCondition::AlwaysChange,
                Action::MoveTo(assets.map.special.find_tile(4)),
            ),
            (
                ActionCondition::ReachedDestination,
                Action::SetInteractMessage(Some(
                    "If its not too much to ask, I would be\nFOREVER thankful if you'd light the fireplace.",
                )),
            ),
            (
                ActionCondition::AlwaysChange,
                Action::SetPlayingAnimation(false),
            ),
            (ActionCondition::AlwaysChange, Action::SetAnimationTime(0.0)),
            (
                ActionCondition::PlayerHasTag(Tag::LightFire),
                Action::SetInteractMessage(None),
            ),
            (ActionCondition::AlwaysChange, Action::SetAnimationTime(0.0)),
            (ActionCondition::AlwaysChange, Action::ChangeAnimation(6)),
            (
                ActionCondition::AlwaysChange,
                Action::SetPlayingAnimation(true),
            ),
            (ActionCondition::AnimationFinish, Action::Noop),
            (
                ActionCondition::AlwaysChange,
                Action::MoveTo(assets.map.special.find_tile(5)),
            ),
            (
                ActionCondition::ReachedDestination,
                Action::ChangeAnimation(5),
            ),
            (ActionCondition::AlwaysChange, Action::SetCollision(false)),
            (
                ActionCondition::AlwaysChange,
                Action::SetPlayingAnimation(true),
            ),
            (
                ActionCondition::AnimationFinish,
                Action::SetPlayingAnimation(false),
            ),
            (
                ActionCondition::Time(2.0),
                Action::GiveTag(Tag::FamilyShouldArrive),
            ),
        ],
        animation: Some(&assets.raincoat_ferret),
        x,
        y,
        name: "Ferret in a raincoat",
        draw_offset: vec2(-16.0, -16.0),
        ..BASE_CHARACTER
    }
}
pub fn mother_ferret<'a>(assets: &'a Assets) -> Character<'a> {
    let (x, y) = assets.map.special.find_tile(1);
    Character {
        draw_pos: vec2(0 as f32, 0 as f32) * 16.0,
        actions: vec![
            (
                ActionCondition::PlayerHasTag(Tag::FamilyShouldArrive),
                Action::Teleport(x, y),
            ),
            (
                ActionCondition::PlayerHasTag(Tag::OpenedDoor2),
                Action::Noop,
            ),
            (ActionCondition::Time(0.8), Action::Noop),
            (
                ActionCondition::Dialogue(
                    "Hi! My boys and I were out playing when this\nstorm struck!",
                ),
                Action::Noop,
            ),
            (
                ActionCondition::Dialogue(
                    "They are cold and tired. May we please\ncome inside and ride out the storm?",
                ),
                Action::Noop,
            ),
            (ActionCondition::Time(0.5), Action::ShowScreen(3)),
            (ActionCondition::Time(1.0), Action::HideScreen),
            (
                ActionCondition::AlwaysChange,
                Action::TeleportPlayer(x + 1, y + 1),
            ),
            (ActionCondition::AlwaysChange, Action::Teleport(x, y + 1)),
            (
                ActionCondition::AlwaysChange,
                Action::GiveTag(Tag::ClosedDoor2),
            ),
        ],
        name: "Ferret Mother",
        animation: Some(&assets.mother_ferret),
        x,
        y,
        ..BASE_CHARACTER
    }
}
pub fn child_ferret<'a>(assets: &'a Assets, id: usize) -> Character<'a> {
    let (x, y) = assets.map.special.find_tile(1);
    Character {
        draw_pos: vec2(0 as f32, 0 as f32) * 16.0,
        actions: vec![
            (
                ActionCondition::PlayerHasTag(Tag::FamilyShouldArrive),
                Action::Teleport(x, y),
            ),
            (
                ActionCondition::PlayerHasTag(Tag::ClosedDoor2),
                Action::Teleport(x - 1, y + 1),
            ),
        ],
        name: "Child Ferret",
        animation: Some(&assets.child_ferret[id]),
        has_collision: false,
        draw_offset: vec2(-5.0 + 10.0 * id as f32 + 3.0, 0.0),
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
                ActionCondition::Dialogue("*knock* *knock* *knock*"),
                Action::Noop,
            ),
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
            (
                ActionCondition::PlayerHasTag(Tag::FamilyShouldArrive),
                Action::Noop,
            ),
            (
                ActionCondition::Dialogue("*knock* *knock* *knock*"),
                Action::Noop,
            ),
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
                Action::GiveTag(Tag::OpenedDoor2),
            ),
            (ActionCondition::AlwaysChange, Action::ShowScreen(2)),
            (
                ActionCondition::PlayerHasTag(Tag::ClosedDoor2),
                Action::SetAnimationTime(0.0),
            ),
        ],
        name: "Door",
        animation: Some(&assets.door),
        x,
        y,
        ..BASE_CHARACTER
    }
}
#[allow(dead_code)]
pub fn test_character<'a>((x, y): (usize, usize), assets: &'a Assets) -> Character<'a> {
    Character {
        draw_pos: vec2(x as f32, y as f32) * 16.0,
        actions: vec![(
            ActionCondition::AlwaysChange,
            Action::MoveTo(assets.map.special.find_tile(0)),
        )],
        animation: Some(&assets.raincoat_ferret),
        x,
        y,
        draw_over: false,
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
            (
                ActionCondition::AlwaysChange,
                Action::GiveTag(Tag::LightFire),
            ),
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
            font_size: (10.0 * ctx.scale_factor) as u16,
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
