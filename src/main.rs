use std::time::Duration;

use bevy::{app::AppExit, prelude::*, window::close_on_esc};

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

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct ExitButton;

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
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::StartMenu)
                .with_system(close_on_esc)
                .with_system(button_visual_interact)
                .with_system(on_start_button.run_if(button_interact::<StartButton>))
                .with_system(on_exit_button.run_if(button_interact::<ExitButton>))
                .into(),
        )
        .add_exit_system(GameState::StartMenu, despawn_with::<StartMenu>)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .with_system(menu_on_esc)
                .into(),
        )
        .add_startup_system(setup_camera)
        .run();
}

fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

/// Spawn the start menu ui
fn setup_start_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_style = Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(8.0)),
        margin: UiRect::all(Val::Px(4.0)),
        flex_grow: 1.0,
        ..Default::default()
    };

    let button_textstyle = TextStyle {
        font: asset_server.load("comic.ttf"),
        font_size: 36.0,
        color: Color::BLACK,
    };

    let menu = commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::rgb(0.5, 0.5, 0.5)),
            style: Style {
                size: Size::new(Val::Auto, Val::Auto),
                margin: UiRect::all(Val::Auto),
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::ColumnReverse,
                //align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(StartMenu)
        .id();

    let start_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style.clone(),
            ..Default::default()
        })
        .with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::from_section("Start Game", button_textstyle.clone()),
                ..Default::default()
            });
        })
        .insert(StartButton)
        .id();

    let exit_button = commands
        .spawn_bundle(ButtonBundle {
            style: button_style,
            ..Default::default()
        })
        .with_children(|btn| {
            btn.spawn_bundle(TextBundle {
                text: Text::from_section("Exit Game", button_textstyle.clone()),
                ..Default::default()
            });
        })
        .insert(ExitButton)
        .id();

    commands
        .entity(menu)
        .push_children(&[start_button, exit_button]);
}

fn button_interact<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    for interaction in &query {
        if *interaction == Interaction::Clicked {
            return true;
        }
    }

    false
}

fn button_visual_interact(
    mut query: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut colour) in &mut query {
        match interaction {
            Interaction::Clicked => {
                *colour = UiColor(Color::rgb(0.75, 0.75, 0.75));
            }
            Interaction::Hovered => {
                *colour = UiColor(Color::rgb(0.8, 0.8, 0.8));
            }
            Interaction::None => {
                *colour = UiColor(Color::rgb(1.0, 1.0, 1.0));
            }
        }
    }
}

fn on_start_button(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Playing))
}

fn on_exit_button(mut exit_writer: EventWriter<AppExit>) {
    exit_writer.send(AppExit);
}

fn menu_on_esc(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::StartMenu))
    }
}
