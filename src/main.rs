use bevy::prelude::*;
use game::game::GamePlugin;
pub mod game;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();    
}
