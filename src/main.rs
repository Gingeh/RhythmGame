use std::time::Duration;

mod utils;
use utils::*;

use bevy::prelude::*;
use bevy::window::close_on_esc;

use iyes_loopless::prelude::*;
// Heavy code reuse from https://github.com/IyesGames/iyes_loopless/blob/main/examples/menu.rs

/// The game's state.. duh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    StartMenu,
    Playing,
    GameOverMenu,
}

/// Marker component for entities used in the start menu
#[derive(Component)]
struct StartMenu;

/// Where all the magic happens
fn main() {
    // Stage for anything that runs on a fixed timestep (i.e. update functions)
    let mut fixedupdate = SystemStage::parallel();

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Rhythm Game".into(),
            width: 450.0,
            height: 700.0,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_loopless_state(GameState::StartMenu)
        .add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_millis(125), fixedupdate),
        )
        .add_enter_system(GameState::StartMenu, setup_start_menu)
        .add_exit_system(GameState::StartMenu, despawn_with::<StartMenu>)
        .run();
}

/// Spawn the start menu ui
fn setup_start_menu() {
    todo!()
}