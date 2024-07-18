mod game_logic;
mod networking;
mod systems;

use std::f32::consts::PI;

use crate::game_logic::GameLogicError;
use crate::networking::post_office::spawn_post_office;
use crate::systems::board_plugin::{
    BoardPlugin,
    CardsRelativePosition,
    PetalsRelativePosition,
    PlayerRelativePos,
    StateLabel,
    StateStringPicker,
};
use crate::systems::picker::{ Pickable, PickerButton, PickerPlugin };
use bevy::app::AppExit;
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use bevy::text::TextStyle;
use bevy::ui::PositionType;
use bevy::DefaultPlugins;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tokio_tasks::{ TaskContext, TokioTasksPlugin, TokioTasksRuntime };
use bevy_tweening::TweeningPlugin;
use furuyoni_lib::rules::player_actions::BasicAction;
use systems::board_plugin::{ CardInspectPosition, DeckObject, HandObject, Spread };
use thiserror::Error;
use tokio::net::TcpStream;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Failed to connect to the server.")] ConnectionFailed(tokio::io::Error),
    #[error("{0}")] GameLogicError(#[from] GameLogicError),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PickerPlugin)
        .add_plugins(BoardPlugin)
        .add_plugins(TokioTasksPlugin::default())
        .add_plugins(TweeningPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (setup, spawn_logic_thread))
        // .add_systems(Startup, load_scene)
        .run();
}

pub(crate) fn spawn_logic_thread(runtime: ResMut<TokioTasksRuntime>) {
    runtime.spawn_background_task(|ctx| async move {
        let result = run_logic_thread(ctx.clone()).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                error!("{e}");
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.send_event(AppExit::Success);
                }).await;
            }
        }
    });
}

async fn run_logic_thread(ctx: TaskContext) -> Result<(), Error> {
    let socket = TcpStream::connect("127.0.0.1:4255").await.map_err(|e|
        Error::ConnectionFailed(e)
    )?;

    let (player_to_game_requester, player_to_game_responder, post_office_task) =
        spawn_post_office(socket);

    let ret = game_logic::run_game(player_to_game_responder, ctx).await;

    post_office_task.abort();

    ret?;
    Ok(())
}

fn _load_scene(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
    let ff: Handle<Font> = asset_server.load("fonts/Fira_Sans/FiraSans-Regular.ttf");
    std::mem::forget(ff);

    scene_spawner.spawn_dynamic(asset_server.load("scenes/main_scene.scn.ron"));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.resolution.set(1920.0, 1080.0);
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/Fira_Sans/FiraSans-Regular.ttf");

    let mut spawn_label = |l, t, str: &str, picker| {
        commands.spawn((
            TextBundle::from_sections([
                TextSection::new(str.to_string() + ": ", TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    ..default()
                }),
                TextSection::new("", TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: GREEN.into(),
                }),
            ])
                .with_text_justify(JustifyText::Left)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(t),
                    left: Val::Percent(l),
                    ..default()
                }),
            StateLabel::new(1, picker),
        ));
    };

    const LH: f32 = 6.0;
    spawn_label(
        10.0,
        10.0,
        "Life",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Life(PlayerRelativePos::Opponent))
    );
    spawn_label(
        10.0,
        10.0 + LH * 1.0,
        "Flare",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Flare(PlayerRelativePos::Opponent))
    );

    spawn_label(
        10.0,
        10.0 + LH * 2.0,
        "Aura",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Aura(PlayerRelativePos::Opponent))
    );

    spawn_label(
        10.0,
        10.0 + LH * 3.0,
        "Vigor",
        StateStringPicker::Vigor(PlayerRelativePos::Opponent)
    );

    spawn_label(
        18.0,
        20.0 + LH * 3.0,
        "Deck",
        StateStringPicker::CardsCount(CardsRelativePosition::Deck(PlayerRelativePos::Opponent))
    );

    spawn_label(
        83.0,
        70.0,
        "Life",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Life(PlayerRelativePos::Me))
    );

    spawn_label(
        83.0,
        70.0 + LH * 1.0,
        "Flare",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Flare(PlayerRelativePos::Me))
    );

    spawn_label(
        83.0,
        70.0 + LH * 2.0,
        "Aura",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Aura(PlayerRelativePos::Me))
    );
    spawn_label(83.0, 70.0 + LH * 3.0, "Vigor", StateStringPicker::Vigor(PlayerRelativePos::Me));
    spawn_label(
        75.0,
        60.0,
        "Deck",
        StateStringPicker::CardsCount(CardsRelativePosition::Deck(PlayerRelativePos::Me))
    );

    spawn_label(85.0, 20.0, "Turn", StateStringPicker::Turn);
    spawn_label(
        50.0,
        40.0,
        "Distance",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Distance)
    );
    spawn_label(
        50.0,
        40.0 + LH * 1.0,
        "Dust",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Dust)
    );

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    // center button
                    margin: UiRect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    right: Val::Percent(20.0),
                    bottom: Val::Percent(20.0),
                    ..default()
                },
                background_color: Color::srgb(125.0 / 256.0, 13.0 / 256.0, 40.0 / 256.0).into(),
                ..default()
            },
            PickerButton {
                pickable: Pickable::Cancel,
            },
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("Cancel", TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                })
            );
        });

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    // center button
                    margin: UiRect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    right: Val::Percent(20.0),
                    bottom: Val::Percent(20.0),
                    ..default()
                },
                background_color: Color::srgb(125.0 / 256.0, 13.0 / 256.0, 40.0 / 256.0).into(),
                ..default()
            },
            PickerButton {
                pickable: Pickable::EndMainPhase,
            },
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section("End Main", TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                })
            );
        });

    commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(155.0),
                height: Val::Px(58.0),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: Val::Percent(82.5),
                top: Val::Percent(87.4),
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::srgba(0.2, 0.5, 0.3, 0.3).into(),
            border_color: BorderColor(Color::WHITE),

            ..default()
        },
        PickerButton {
            pickable: Pickable::Vigor,
        },
    ));

    let mut spawn_ba_button = |bottom, right, str: &str, action| {
        commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        // center button
                        margin: UiRect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        right: Val::Percent(right),
                        bottom: Val::Percent(bottom),
                        ..default()
                    },
                    background_color: Color::srgb(0.2, 0.5, 0.3).into(),
                    ..default()
                },
                PickerButton {
                    pickable: Pickable::BasicAction(action),
                },
            ))
            .with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(str, TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                    })
                );
            });
    };

    spawn_ba_button(20.0, 46.0, "Forward", BasicAction::MoveForward);
    spawn_ba_button(10.0, 46.0, "Backward", BasicAction::MoveBackward);
    spawn_ba_button(20.0, 33.0, "Focus", BasicAction::Focus);
    spawn_ba_button(10.0, 33.0, "Recover", BasicAction::Recover);

    // spawn deck position indicators.
    const DECK_CARDS_SCALE: Vec3 = Vec3::splat(0.7);
    commands.spawn((
        Name::new("Deck(Opponent)"),
        TransformBundle::from_transform(
            Transform::from_xyz(800.0, -80.0, 0.0).with_scale(DECK_CARDS_SCALE)
        ),
        DeckObject::new(PlayerRelativePos::Me),
    ));

    commands.spawn((
        Name::new("Deck(Opponent)"),
        TransformBundle::from_transform(
            Transform::from_xyz(-800.0, 80.0, 0.0)
                .with_rotation(Quat::from_rotation_z(PI))
                .with_scale(DECK_CARDS_SCALE)
        ),
        DeckObject::new(PlayerRelativePos::Opponent),
    ));

    // spawn card hands.
    commands.spawn((
        Name::new("Hand(Me)"),
        SpatialBundle::from_transform(
            Transform::from_xyz(0.0, -450.0, 200.0).with_scale(DECK_CARDS_SCALE)
        ),
        HandObject::new(PlayerRelativePos::Me),
        Spread::new(600.0, 7),
    ));
    commands.spawn((
        Name::new("Hand(Opponent)"),
        SpatialBundle::from_transform(
            Transform::from_xyz(0.0, 450.0, 0.0)
                .with_rotation(Quat::from_rotation_z(PI))
                .with_scale(DECK_CARDS_SCALE)
        ),
        HandObject::new(PlayerRelativePos::Opponent),
        Spread::new(600.0, 7),
    ));

    const INSPECTOR_SCALE: Vec3 = Vec3::splat(2.0);

    commands.spawn((
        Name::new("InspectorPosition"),
        TransformBundle::from_transform(
            Transform::from_xyz(700.0, -20.0, 100.0).with_scale(INSPECTOR_SCALE)
        ),
        CardInspectPosition {},
    ));
}
