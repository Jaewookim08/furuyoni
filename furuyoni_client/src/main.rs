mod networking;
mod systems;

use crate::networking::{post_office, ClientConnectionReader, ClientConnectionWriter};
use crate::systems::display_board::{display_board, StateLabel, StateStringPicker};
use bevy::prelude::*;
use bevy::text::TextStyle;
use bevy::ui::PositionType;
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::*;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameRequest, GameToPlayerRequestData, GameToPlayerRequestDataFrame,
    GameToPlayerResponse, GameToPlayerResponseFrame, PlayerMessageFrame, PlayerResponse,
    PlayerResponseFrame, PlayerToGameRequest, ResponseMainPhaseAction,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::IntoMessageMap;
use furuyoni_lib::net::{Requester, Responser};
use furuyoni_lib::players::{CliPlayer, Player};
use furuyoni_lib::rules::{
    Phase, PlayerPos, ViewableOpponentState, ViewablePlayerState, ViewablePlayerStates,
    ViewableSelfState, ViewableState,
};
use iyes_loopless::prelude::*;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

#[derive(Resource, Debug)]
pub struct GameState {
    state: ViewableState,
}

// Todo: remove. GameState should not be constructed in client.
impl Default for GameState {
    fn default() -> Self {
        Self {
            state: ViewableState {
                turn_number: 0,
                turn_player: PlayerPos::P1,
                phase: Phase::Beginning,
                distance: 0,
                dust: 0,
                player_states: ViewablePlayerStates::new(
                    ViewablePlayerState::SelfState(ViewableSelfState {
                        hands: vec![],
                        deck_count: 0,
                        enhancements: vec![],
                        played_pile: vec![],
                        discard_pile: vec![],
                        vigor: 0,
                        aura: 0,
                        life: 0,
                        flare: 0,
                    }),
                    ViewablePlayerState::Opponent(ViewableOpponentState {
                        hand_count: 0,
                        deck_count: 0,
                        enhancements: vec![],
                        played_pile: vec![],
                        discard_pile_count: 0,
                        vigor: 0,
                        aura: 0,
                        life: 0,
                        flare: 0,
                    }),
                ),
            },
        }
    }
}

fn hello_world() {
    println!("Hello world!")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    App::new()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_system(display_board)
        .add_startup_system(setup)
        .run();

    // let socket = TcpStream::connect("127.0.0.1:4255").await?;
    //
    // let (player_to_game_requester, mut player_to_game_responser, post_office_task) =
    //     spawn_post_office(socket);
    //
    // let player = CliPlayer {};
    //
    // run_responser(player, player_to_game_responser).await;
    //
    // post_office_task.abort();
    Ok(())
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/Fira_Sans/FiraSans-Regular.ttf");
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Hello world!: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    ..default()
                },
            ),
            TextSection::new(
                "Empty",
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::GREEN,
                },
            ),
        ])
        .with_text_alignment(TextAlignment::CENTER_LEFT)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(100.0),
                left: Val::Px(100.0),
                ..default()
            },
            ..default()
        }),
        StateLabel::new(1, StateStringPicker::Distance),
    ));
}

async fn run_responser(
    mut player: impl Player,
    mut responser: impl Responser<PlayerResponseFrame, Request = GameRequest>,
) {
    loop {
        let req = responser.recv().await.unwrap();
        match req {
            GameRequest::RequestData(GameToPlayerRequestDataFrame {
                request_id,
                data: req,
            }) => {
                let response = match req {
                    GameToPlayerRequestData::RequestMainPhaseAction(r) => {
                        let action = player
                            .get_main_phase_action(
                                &r.state,
                                &r.playable_cards,
                                &r.performable_basic_actions,
                                &r.available_basic_action_costs,
                            )
                            .await;

                        PlayerResponse::ResponseMainPhaseAction(ResponseMainPhaseAction { action })
                    }
                };

                responser
                    .response(PlayerResponseFrame::new(request_id, response))
                    .unwrap();
            }
            GameRequest::Notify(n) => match n {},
        }
    }
}

fn spawn_post_office(
    stream: TcpStream,
) -> (
    impl Requester<PlayerToGameRequest, Response = GameToPlayerResponse>,
    impl Responser<PlayerResponseFrame, Request = GameRequest>,
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
        ClientMessageFrame::PlayerMessage(PlayerMessageFrame::Request(request))
    });

    let player_to_game_requester =
        MessageChannel::new(player_to_game_request_sender, game_response_rx);

    let player_to_game_response_sender = client_message_tx
        .clone()
        .with_map(|r| ClientMessageFrame::PlayerMessage(PlayerMessageFrame::Response(r)));

    let player_to_game_responser =
        MessageChannel::new(player_to_game_response_sender, game_request_rx);

    return (
        player_to_game_requester,
        player_to_game_responser,
        post_office_joinhandle,
    );
}
