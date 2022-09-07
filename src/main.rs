use std::time::Duration;

use bevy::{app::AppExit, prelude::*, window::close_on_esc};

use iyes_loopless::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
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

/// Component containing a button's previous interaction state
#[derive(Component)]
struct OldInteraction(Interaction);

#[derive(Component)]
struct Target;

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component, PartialEq, Eq, Clone, Copy)]
enum Column {
    Yellow,
    Red,
    Blue,
    Green,
}

impl Column {
    fn index(&self) -> usize {
        match self {
            Column::Yellow => 0,
            Column::Red => 1,
            Column::Blue => 2,
            Column::Green => 3,
        }
    }
}

impl Distribution<Column> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Column {
        match rng.gen_range(0..4) {
            0 => Column::Yellow,
            1 => Column::Red,
            2 => Column::Blue,
            _ => Column::Green,
        }
    }
}

#[derive(Default)]
struct TextureAtlasHandles {
    crosshairs: Option<Handle<TextureAtlas>>,
    targets: Option<Handle<TextureAtlas>>,
}

#[derive(Default)]
struct NoteAudioHandles {
    yellow: Option<Handle<AudioSource>>,
    red: Option<Handle<AudioSource>>,
    blue: Option<Handle<AudioSource>>,
    green: Option<Handle<AudioSource>>,
}

#[derive(Default)]
struct Scoreboard {
    pub score: i32,
    pub combo: i32,
}

impl Scoreboard {
    fn hit(&mut self) {
        if self.combo < 5 {
            self.combo += 1;
        }
        self.score += self.combo;
    }

    fn miss(&mut self) {
        self.combo = 0;
        self.score -= self.combo + 1;
    }
}

struct TargetHitEvent(Column);

struct TargetMissEvent(Column);

/// Where all the magic happens
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Rhythm Game".into(),
            width: 450.0,
            height: 700.0,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_event::<TargetHitEvent>()
        .add_event::<TargetMissEvent>()
        // Set GameState::StartMenu as the default state
        .add_loopless_state(GameState::StartMenu)
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
                .with_system(update_targets)
                .with_system(shoot_targets)
                .with_system(play_hit_sound)
                .with_system(update_scoreboard)
                .into(),
        )
        .add_stage_before(
            CoreStage::Update,
            "SpawnTargets",
            FixedTimestepStage::new(Duration::from_millis(350)).with_stage(SystemStage::single(
                spawn_targets.run_in_state(GameState::Playing),
            )),
        )
        // Despawn the entire game when it is exited
        .add_exit_system(GameState::Playing, despawn_with::<Game>)
        // Spawn the camera (for the game and for the UI)
        .add_startup_system(setup_camera)
        .init_resource::<TextureAtlasHandles>()
        .init_resource::<NoteAudioHandles>()
        .init_resource::<Scoreboard>()
        .add_startup_system(load_assets)
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

fn load_assets(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut atlas_handles: ResMut<TextureAtlasHandles>,
    mut audio_handles: ResMut<NoteAudioHandles>,
) {
    let crosshair_texture_handle = asset_server.load("textures/crosshairs.png");
    let crosshair_texture_atlas =
        TextureAtlas::from_grid(crosshair_texture_handle, Vec2::new(64.0, 64.0), 4, 1);
    let crosshair_atlas_handle = texture_atlases.add(crosshair_texture_atlas);

    let target_texture_handle = asset_server.load("textures/targets.png");
    let target_texture_atlas =
        TextureAtlas::from_grid(target_texture_handle, Vec2::new(64.0, 64.0), 4, 1);
    let target_atlas_handle = texture_atlases.add(target_texture_atlas);

    atlas_handles.crosshairs = Some(crosshair_atlas_handle);
    atlas_handles.targets = Some(target_atlas_handle);

    audio_handles.yellow = Some(asset_server.load("sounds/notes/yellow.ogg"));
    audio_handles.red = Some(asset_server.load("sounds/notes/red.ogg"));
    audio_handles.blue = Some(asset_server.load("sounds/notes/blue.ogg"));
    audio_handles.green = Some(asset_server.load("sounds/notes/green.ogg"));
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
        .insert(OldInteraction(Interaction::None))
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
        .insert(OldInteraction(Interaction::None))
        .id();

    commands
        .entity(menu)
        .push_children(&[start_button, exit_button]);
}

/// Returns true if any buttons with the given component are being pressed
fn button_interact<B: Component>(
    mut interactions: Query<
        (&Interaction, &mut OldInteraction),
        (Changed<Interaction>, With<Button>, With<B>),
    >,
) -> bool {
    for (new_interaction, mut old_interaction) in &mut interactions {
        if *new_interaction == Interaction::Hovered && old_interaction.0 == Interaction::Clicked {
            return true;
        }
        old_interaction.0 = *new_interaction;
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
fn setup_game(
    mut commands: Commands,
    atlas_handles: Res<TextureAtlasHandles>,
    asset_server: Res<AssetServer>,
) {
    let atlas_handle = atlas_handles.crosshairs.as_ref().unwrap();

    for column in [Column::Yellow, Column::Red, Column::Blue, Column::Green] {
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_xyz((column.index() as f32) * 90.0 - 135.0, -305.0, 0.0)
                    .with_scale(Vec3::splat(0.3)),
                sprite: TextureAtlasSprite {
                    index: column.index(),
                    custom_size: Some(Vec2::splat(200.0)),
                    ..Default::default()
                },
                texture_atlas: atlas_handle.clone(),
                ..Default::default()
            })
            .insert(Game)
            .insert(column);
    }

    let score_textstyle = TextStyle {
        font: asset_server.load("fonts/comic.ttf"),
        font_size: 36.0,
        color: Color::WHITE,
    };

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_sections([
                TextSection {
                    value: "Score: ".into(),
                    style: score_textstyle.clone(),
                },
                TextSection {
                    value: "0".into(),
                    style: score_textstyle.clone(),
                },
            ]),
            transform: Transform::from_xyz(-200.0, 300.0, 0.0),
            ..Default::default()
        })
        .insert(Game)
        .insert(ScoreDisplay);
}

/// Exit to the start menu if the player pressed escape
fn menu_on_esc(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(GameState::StartMenu))
    }
}

fn spawn_targets(mut commands: Commands, atlas_handles: Res<TextureAtlasHandles>) {
    let mut rng = rand::thread_rng();
    let column = rng.gen::<Column>();

    let atlas_handle = atlas_handles.targets.as_ref().unwrap();

    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform::from_xyz((column.index() as f32) * 90.0 - 135.0, 400.0, 0.0)
                .with_scale(Vec3::splat(0.3)),
            sprite: TextureAtlasSprite {
                index: column.index(),
                custom_size: Some(Vec2::splat(200.0)),
                ..Default::default()
            },
            texture_atlas: atlas_handle.clone(),
            ..Default::default()
        })
        .insert(Game)
        .insert(Target)
        .insert(column);
}

fn update_targets(
    mut commands: Commands,
    mut targets: Query<(Entity, &mut Transform, &Column), With<Target>>,
    time: Res<Time>,
    mut miss_event_writer: EventWriter<TargetMissEvent>,
    mut score: ResMut<Scoreboard>,
) {
    for (target, mut transform, column) in targets.iter_mut() {
        if transform.translation.y < -350.0 {
            commands.entity(target).despawn();
            miss_event_writer.send(TargetMissEvent(*column));
            score.miss();
        } else {
            transform.translation.y -= 150.0 * time.delta_seconds();
        }
    }
}

fn shoot_targets(
    mut commands: Commands,
    targets: Query<(Entity, &Transform, &Column), With<Target>>,
    input: Res<Input<KeyCode>>,
    mut hit_event_writer: EventWriter<TargetHitEvent>,
    mut score: ResMut<Scoreboard>,
) {
    if input.any_just_pressed([KeyCode::A, KeyCode::H]) {
        targets
            .iter()
            .filter(|(_, transform, column)| {
                *column == &Column::Yellow && transform.translation.y <= -280.0
            })
            .for_each(|(target, _, column)| {
                commands.entity(target).despawn();
                hit_event_writer.send(TargetHitEvent(*column));
                score.hit();
            });
    }

    if input.any_just_pressed([KeyCode::S, KeyCode::J]) {
        targets
            .iter()
            .filter(|(_, transform, column)| {
                *column == &Column::Red && transform.translation.y <= -280.0
            })
            .for_each(|(target, _, column)| {
                commands.entity(target).despawn();
                hit_event_writer.send(TargetHitEvent(*column));
                score.hit();
            });
    }

    if input.any_just_pressed([KeyCode::D, KeyCode::K]) {
        targets
            .iter()
            .filter(|(_, transform, column)| {
                *column == &Column::Blue && transform.translation.y <= -280.0
            })
            .for_each(|(target, _, column)| {
                commands.entity(target).despawn();
                hit_event_writer.send(TargetHitEvent(*column));
                score.hit();
            });
    }

    if input.any_just_pressed([KeyCode::F, KeyCode::L]) {
        targets
            .iter()
            .filter(|(_, transform, column)| {
                *column == &Column::Green && transform.translation.y <= -280.0
            })
            .for_each(|(target, _, column)| {
                commands.entity(target).despawn();
                hit_event_writer.send(TargetHitEvent(*column));
                score.hit();
            });
    }

    //FIXME: Holy code duplication, Batman!
}

fn play_hit_sound(
    mut hit_event_reader: EventReader<TargetHitEvent>,
    audio: Res<Audio>,
    audio_handles: Res<NoteAudioHandles>,
) {
    for TargetHitEvent(column) in hit_event_reader.iter() {
        if let Some(audio_handle) = match column {
            Column::Yellow => &audio_handles.yellow,
            Column::Red => &audio_handles.red,
            Column::Blue => &audio_handles.blue,
            Column::Green => &audio_handles.green,
        } {
            audio.play(audio_handle.clone());
        };
    }
}

fn update_scoreboard(
    score: Res<Scoreboard>,
    mut score_text_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    if score.is_changed() {
        for mut score_text in score_text_query.iter_mut() {
            score_text.sections[1].value = score.score.to_string()
        }
    }
}
