use std::time::Duration;

use bevy::{app::AppExit, prelude::*, window::close_on_esc};

use iyes_loopless::prelude::*;
// Heavy code reuse from https://github.com/IyesGames/iyes_loopless/blob/main/examples/menu.rs

/// The game's states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    StartMenu,
    Playing,
    GameOverMenu,
}

/// Marker component for entities used in the start menu
#[derive(Component)]
struct StartMenu;

/// Marker component for entities used in the game
#[derive(Component)]
struct Game;

/// Marker component for the start button
#[derive(Component)]
struct StartButton;

/// Marker component for the exit button
#[derive(Component)]
struct ExitButton;

#[derive(Default)]
struct TextureAtlasHandles {
    crosshairs: Option<Handle<TextureAtlas>>,
    targets: Option<Handle<TextureAtlas>>,
}

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

        // Set GameState::StartMenu as the default state
        .add_loopless_state(GameState::StartMenu)

        // Register the FixedUpdate stage to run every 125ms
        .add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_millis(125), fixedupdate),
        )

        // Setup the start menu when GameState::StartMenu is entered
        .add_enter_system(GameState::StartMenu, setup_start_menu)

        .add_system_set(
            ConditionSet::new()
                // While the start menu is visible..
                .run_in_state(GameState::StartMenu)
                // Quit the game if the player presses escape
                .with_system(close_on_esc)
                // Change the colour of the buttons when the player interacts with them
                .with_system(button_visual_interact)
                // Run the associated code when the buttons are clicked
                .with_system(on_start_button.run_if(button_interact::<StartButton>))
                .with_system(on_exit_button.run_if(button_interact::<ExitButton>))
                .into(),
        )

        // Despawn the entire start menu when it is exited
        .add_exit_system(GameState::StartMenu, despawn_with::<StartMenu>)

        // Setup the game when GameState::Playing is entered
        .add_enter_system(GameState::Playing, setup_game)

        .add_system_set(
            ConditionSet::new()
                // While the game is running
                .run_in_state(GameState::Playing)
                // Exit to the menu when the player presses escape
                .with_system(menu_on_esc)
                .into(),
        )

        // Despawn the entire game when it is exited
        .add_exit_system(GameState::Playing, despawn_with::<Game>)

        // Spawn the camera (for the game and for the UI)
        .add_startup_system(setup_camera)

        .init_resource::<TextureAtlasHandles>()
        .add_startup_system(load_textures)
        .run();
}

/// Recursively despawns every entity with a given component
fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Spawn a 2D camera
fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn load_textures(asset_server: Res<AssetServer>, mut texture_atlases: ResMut<Assets<TextureAtlas>>, mut handles: ResMut<TextureAtlasHandles>) {
    let crosshair_texture_handle = asset_server.load("textures/crosshairs.png");
    let crosshair_texture_atlas = TextureAtlas::from_grid(crosshair_texture_handle, Vec2::new(64.0, 64.0), 4, 1);
    let crosshair_atlas_handle = texture_atlases.add(crosshair_texture_atlas);
    
    let target_texture_handle = asset_server.load("textures/targets.png");
    let target_texture_atlas = TextureAtlas::from_grid(target_texture_handle, Vec2::new(64.0, 64.0), 4, 1);
    let target_atlas_handle = texture_atlases.add(target_texture_atlas);

    handles.crosshairs = Some(crosshair_atlas_handle);
    handles.targets = Some(target_atlas_handle);
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
        font: asset_server.load("fonts/comic.ttf"),
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

/// Returns true if any buttons with the given component are being pressed
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

/// Sets the colour of every button based on player interaction
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

/// Starts the game
fn on_start_button(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Playing))
}

/// Exits the game
fn on_exit_button(mut exit_writer: EventWriter<AppExit>) {
    exit_writer.send(AppExit);
}

/// Sets up the game
fn setup_game(mut commands: Commands, atlas_handles: Res<TextureAtlasHandles>) {
    let atlas_handle = atlas_handles.crosshairs.as_ref().unwrap();

    commands.spawn_bundle(SpriteSheetBundle {
        transform: Transform::from_xyz(-135.0, -305.0, 0.0).with_scale(Vec3::splat(0.3)),
        sprite: TextureAtlasSprite {
            index: 0,
            custom_size: Some(Vec2::splat(200.0)),
            ..Default::default()
        },
        texture_atlas: atlas_handle.clone(),
        ..Default::default()
    }).insert(Game);

    commands.spawn_bundle(SpriteSheetBundle {
        transform: Transform::from_xyz(-45.0, -305.0, 0.0).with_scale(Vec3::splat(0.3)),
        sprite: TextureAtlasSprite {
            index: 1,
            custom_size: Some(Vec2::splat(200.0)),
            ..Default::default()
        },
        texture_atlas: atlas_handle.clone(),
        ..Default::default()
    }).insert(Game);

    commands.spawn_bundle(SpriteSheetBundle {
        transform: Transform::from_xyz(45.0, -305.0, 0.0).with_scale(Vec3::splat(0.3)),
        sprite: TextureAtlasSprite {
            index: 2,
            custom_size: Some(Vec2::splat(200.0)),
            ..Default::default()
        },
        texture_atlas: atlas_handle.clone(),
        ..Default::default()
    }).insert(Game);

    commands.spawn_bundle(SpriteSheetBundle {
        transform: Transform::from_xyz(135.0, -305.0, 0.0).with_scale(Vec3::splat(0.3)),
        sprite: TextureAtlasSprite {
            index: 3,
            custom_size: Some(Vec2::splat(200.0)),
            ..Default::default()
        },
        texture_atlas: atlas_handle.clone(),
        ..Default::default()
    }).insert(Game);
}

/// Exit to the start menu if the player pressed escape
fn menu_on_esc(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::StartMenu))
    }
}
