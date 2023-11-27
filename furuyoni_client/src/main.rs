mod game_logic;
mod networking;
mod systems;

use crate::game_logic::GameLogicError;
use crate::networking::post_office::spawn_post_office;
use crate::systems::board_system::{
    PlayerRelativePos, PlayerValuePicker, PlayerValuePickerType, StateLabel, StateStringPicker,
};
use crate::systems::picker::{BasicActionButton, PickerPlugin, SkipButton};
use bevy::prelude::*;
use bevy::text::TextStyle;
use bevy::ui::PositionType;
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::*;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use furuyoni_lib::net::frames::*;
use furuyoni_lib::net::message_sender::IntoMessageMap;
use furuyoni_lib::player_actions::BasicAction;
use thiserror::Error;
use tokio::net::TcpStream;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Failed to connect to the server.")]
    ConnectionFailed(tokio::io::Error),
    #[error("{0}")]
    GameLogicError(#[from] GameLogicError),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PickerPlugin)
        .add_plugins(TokioTasksPlugin::default())
        .add_plugins(EditorPlugin::default())
        .add_systems(Startup, (setup, spawn_async_tasks))
        // .add_systems(Startup, load_scene)
        .run();
}

pub fn spawn_async_tasks(runtime: ResMut<TokioTasksRuntime>) {
    runtime.spawn_background_task(|ctx| async move {
        let socket = TcpStream::connect("127.0.0.1:4255")
            .await
            .map_err(|e| Error::ConnectionFailed(e))?;

        let (player_to_game_requester, player_to_game_responder, post_office_task) =
            spawn_post_office(socket);

        game_logic::run_game(player_to_game_responder, ctx).await?;

        post_office_task.abort();

        Ok::<(), Error>(())
    });
}

fn load_scene(asset_server: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
    let ff: Handle<Font> = asset_server.load("fonts/Fira_Sans/FiraSans-Regular.ttf");
    std::mem::forget(ff);

    scene_spawner.spawn_dynamic(asset_server.load("scenes/main_scene.scn.ron"));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/Fira_Sans/FiraSans-Regular.ttf");

    let mut spawn_label = |l, t, str: &str, picker| {
        commands.spawn((
            TextBundle::from_sections([
                TextSection::new(
                    str.to_string() + ": ",
                    TextStyle {
                        font: font.clone(),
                        font_size: 50.0,
                        ..default()
                    },
                ),
                TextSection::new(
                    "",
                    TextStyle {
                        font: font.clone(),
                        font_size: 50.0,
                        color: Color::GREEN,
                    },
                ),
            ])
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(t),
                left: Val::Percent(l),
                ..default()
            }),
            StateLabel::new(1, picker),
        ));
    };

    const LH: f32 = 6.;
    spawn_label(
        10.,
        10.,
        "Life",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Opponent,
            PlayerValuePickerType::Life,
        )),
    );
    spawn_label(
        10.,
        10. + LH * 1.,
        "Flare",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Opponent,
            PlayerValuePickerType::Flare,
        )),
    );

    spawn_label(
        10.,
        10. + LH * 2.,
        "Aura",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Opponent,
            PlayerValuePickerType::Aura,
        )),
    );

    spawn_label(
        10.,
        10. + LH * 3.,
        "Vigor",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Opponent,
            PlayerValuePickerType::Vigor,
        )),
    );

    spawn_label(
        80.,
        70.,
        "Life",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Me,
            PlayerValuePickerType::Life,
        )),
    );

    spawn_label(
        80.,
        70. + LH * 1.,
        "Flare",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Me,
            PlayerValuePickerType::Flare,
        )),
    );

    spawn_label(
        80.,
        70. + LH * 2.,
        "Aura",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Me,
            PlayerValuePickerType::Aura,
        )),
    );
    spawn_label(
        80.,
        70. + LH * 3.,
        "Vigor",
        StateStringPicker::PlayerValue(PlayerValuePicker::new(
            PlayerRelativePos::Me,
            PlayerValuePickerType::Vigor,
        )),
    );
    spawn_label(85., 20., "Turn", StateStringPicker::Turn);
    spawn_label(50., 40., "Distance", StateStringPicker::Distance);
    spawn_label(50., 40. + LH * 1., "Dust", StateStringPicker::Dust);

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
                    left: Val::Percent(21.),
                    top: Val::Percent(20.),
                    ..default()
                },
                background_color: Color::rgb(125. / 256., 13. / 256., 40.0 / 256.).into(),
                ..default()
            },
            SkipButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Skip",
                TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });

    let mut spawn_ba_button = |l, t, str: &str, action| {
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
                        top: Val::Percent(t),
                        left: Val::Percent(l),
                        ..default()
                    },
                    background_color: Color::rgb(0.2, 0.5, 0.3).into(),
                    ..default()
                },
                BasicActionButton { action },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    str,
                    TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ));
            });
    };

    spawn_ba_button(-5., 20., "Forward", BasicAction::MoveForward);
    spawn_ba_button(8., 20., "Backward", BasicAction::MoveBackward);
    spawn_ba_button(-5., 30., "Focus", BasicAction::Focus);
    spawn_ba_button(8., 30., "Recover", BasicAction::Recover);
}
