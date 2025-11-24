use macroquad::prelude::*;

use crate::assets::Assets;

mod assets;
mod utils;

#[macroquad::main("winternight")]
async fn main() {
    let assets = Assets::load();
    loop {
        assets.map.draw(&assets);
        next_frame().await
    }
}
