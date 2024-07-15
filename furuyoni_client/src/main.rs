mod game_logic;
mod networking;
mod systems;

use crate::game_logic::GameLogicError;
use crate::networking::post_office::spawn_post_office;
use crate::systems::board_system::{
    BoardPlugin, CardsRelativePosition, PetalsRelativePosition, PlayerRelativePos, StateLabel,
    StateStringPicker,
};
use crate::systems::picker::{Pickable, PickerButton, PickerPlugin};
use bevy::app::AppExit;
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use bevy::text::TextStyle;
use bevy::ui::PositionType;
use bevy::DefaultPlugins;
use bevy_tokio_tasks::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};
use furuyoni_lib::net::frames::*;
use furuyoni_lib::net::message_sender::IntoMessageMap;
use furuyoni_lib::rules::player_actions::BasicAction;
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
        .add_plugins(BoardPlugin)
        .add_plugins(TokioTasksPlugin::default())
        .add_systems(Startup, (setup, spawn_logic_thread))
        // .add_systems(Startup, load_scene)
        .run();
}

pub(crate) fn spawn_logic_thread(runtime: ResMut<TokioTasksRuntime>) {
    runtime.spawn_background_task(|mut ctx| async move {
        let result = run_logic_thread(ctx.clone()).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                error!("{e}");
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.send_event(AppExit::Success);
                })
                .await;
            }
        }
    });
}

pub(crate) async fn run_logic_thread(ctx: TaskContext) -> Result<(), Error> {
    let socket = TcpStream::connect("127.0.0.1:4255")
        .await
        .map_err(|e| Error::ConnectionFailed(e))?;

    let (player_to_game_requester, player_to_game_responder, post_office_task) =
        spawn_post_office(socket);

    game_logic::run_game(player_to_game_responder, ctx).await?;

    // unreachable.
    // Todo: run_game이 에러난 경우에도 실행되게...
    post_office_task.abort();

    Ok(())
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
                        color: GREEN.into(),
                    },
                ),
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

    const LH: f32 = 6.;
    spawn_label(
        10.,
        10.,
        "Life",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Life(PlayerRelativePos::Opponent)),
    );
    spawn_label(
        10.,
        10. + LH * 1.,
        "Flare",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Flare(PlayerRelativePos::Opponent)),
    );

    spawn_label(
        10.,
        10. + LH * 2.,
        "Aura",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Aura(PlayerRelativePos::Opponent)),
    );

    spawn_label(
        10.,
        10. + LH * 3.,
        "Vigor",
        StateStringPicker::Vigor(PlayerRelativePos::Opponent),
    );

    spawn_label(
        83.,
        70.,
        "Life",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Life(PlayerRelativePos::Me)),
    );

    spawn_label(
        83.,
        70. + LH * 1.,
        "Flare",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Flare(PlayerRelativePos::Me)),
    );

    spawn_label(
        83.,
        70. + LH * 2.,
        "Aura",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Aura(PlayerRelativePos::Me)),
    );
    spawn_label(
        83.,
        70. + LH * 3.,
        "Vigor",
        StateStringPicker::Vigor(PlayerRelativePos::Me),
    );
    spawn_label(
        75.,
        60.,
        "Deck",
        StateStringPicker::CardsCount(CardsRelativePosition::Deck(PlayerRelativePos::Me)),
    );

    spawn_label(85., 20., "Turn", StateStringPicker::Turn);
    spawn_label(
        50.,
        40.,
        "Distance",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Distance),
    );
    spawn_label(
        50.,
        40. + LH * 1.,
        "Dust",
        StateStringPicker::PetalsCount(PetalsRelativePosition::Dust),
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
                    right: Val::Percent(20.),
                    bottom: Val::Percent(20.),
                    ..default()
                },
                background_color: Color::rgb(125. / 256., 13. / 256., 40.0 / 256.).into(),
                ..default()
            },
            PickerButton {
                pickable: Pickable::Cancel,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Cancel",
                TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
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
                    right: Val::Percent(20.),
                    bottom: Val::Percent(20.),
                    ..default()
                },
                background_color: Color::rgb(125. / 256., 13. / 256., 40.0 / 256.).into(),
                ..default()
            },
            PickerButton {
                pickable: Pickable::EndMainPhase,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "End Main",
                TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
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
            background_color: Color::rgba(0.2, 0.5, 0.3, 0.3).into(),
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
                    background_color: Color::rgb(0.2, 0.5, 0.3).into(),
                    ..default()
                },
                PickerButton {
                    pickable: Pickable::BasicAction(action),
                },
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

    spawn_ba_button(20., 46., "Forward", BasicAction::MoveForward);
    spawn_ba_button(10., 46., "Backward", BasicAction::MoveBackward);
    spawn_ba_button(20., 33., "Focus", BasicAction::Focus);
    spawn_ba_button(10., 33., "Recover", BasicAction::Recover);
}
