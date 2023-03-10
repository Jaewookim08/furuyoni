mod networking;
mod systems;

use crate::networking::{post_office, ClientConnectionReader, ClientConnectionWriter};
use crate::systems::display_board::{
    display_board, BoardPlugin, PlayerRelativePos, PlayerValuePicker, PlayerValuePickerType,
    StateLabel, StateStringPicker,
};
use crate::systems::picker::{BasicActionButton, PickerPlugin, SkipButton};
use crate::systems::player;
use crate::systems::player::PlayerPlugin;
use bevy::prelude::*;
use bevy::text::TextStyle;
use bevy::ui::PositionType;
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::*;
use furuyoni_lib::net::frames::*;
use furuyoni_lib::net::message_channel::{MessageChannel, MessageChannelResponseError};
use furuyoni_lib::net::message_sender::IntoMessageMap;
use furuyoni_lib::net::{RequestError, Requester, Responder};
use furuyoni_lib::player_actions::BasicAction;
use furuyoni_lib::players::{CliPlayer, Player};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = TcpStream::connect("127.0.0.1:4255").await?;

    let (player_to_game_requester, player_to_game_responder, post_office_task) =
        spawn_post_office(socket);

    App::new()
        .insert_resource(player::PlayerToGameResponder::new(Box::new(
            player_to_game_responder,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(PickerPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BoardPlugin)
        .add_startup_system(setup)
        // .add_startup_system(load_scene)
        .run();

    // let player = CliPlayer {};

    // run_responder(player, player_to_game_responder).await;

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

    for _ in 0..11 {
        commands.spawn((
            TextBundle::from_sections([
                TextSection::new(
                    "Dust: ",
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
                position: UiRect {
                    top: Val::Px(100.0),
                    left: Val::Px(100.0),
                    ..default()
                },
                ..default()
            }),
            StateLabel::new(
                1,
                StateStringPicker::PlayerValue(PlayerValuePicker::new(
                    PlayerRelativePos::Me,
                    PlayerValuePickerType::Life,
                )),
            ),
        ));
    }

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                    // center button
                    margin: UiRect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgb(0.2, 0.5, 0.3).into(),
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

    for _ in 0..4 {
        commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        // center button
                        margin: UiRect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.2, 0.5, 0.3).into(),
                    ..default()
                },
                BasicActionButton {
                    action: BasicAction::Focus,
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "FC",
                    TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ));
            });
    }
}

fn spawn_post_office(
    stream: TcpStream,
) -> (
    impl Requester<PlayerToGameRequest, Response = GameToPlayerResponse, Error = RequestError>
        + Send
        + Sync,
    impl Responder<
            PlayerToGameResponseFrame,
            Request = GameToPlayerRequest,
            Error = MessageChannelResponseError,
        > + Send
        + Sync,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ClientConnectionReader::new(read_half);
    let writer = ClientConnectionWriter::new(write_half);

    let (game_request_tx, game_request_rx) = tokio::sync::mpsc::channel(20);
    let (game_response_tx, game_response_rx) = tokio::sync::mpsc::channel(20);

    let (client_message_tx, client_message_rx) = tokio::sync::mpsc::channel(20);

    let post_office_joinhandle = tokio::spawn(async {
        tokio::select!(
            res = post_office::receive_posts(reader, game_request_tx, game_response_tx) =>
                println!("receive_posts has ended with result: {:?}", res),
            () = post_office::handle_send_requests(client_message_rx, writer) =>
                println!("handle_send_request has ended."),
        );
    });

    let player_to_game_request_sender = client_message_tx.clone().with_map(|request| {
        ClientMessageFrame::PlayerToGameMessage(PlayerToGameMessage::Request(request))
    });

    let player_to_game_requester =
        MessageChannel::new(player_to_game_request_sender, game_response_rx);

    let player_to_game_response_sender = client_message_tx
        .clone()
        .with_map(|r| ClientMessageFrame::PlayerToGameMessage(PlayerToGameMessage::Response(r)));

    let player_to_game_responder =
        MessageChannel::new(player_to_game_response_sender, game_request_rx);

    return (
        player_to_game_requester,
        player_to_game_responder,
        post_office_joinhandle,
    );
}
