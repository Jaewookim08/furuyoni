#![feature(try_trait_v2)]
#![feature(adt_const_params)]
#![feature(let_chains)]
extern crate furuyoni_lib;

mod game;

mod game_watcher;
mod main_channels;
mod networking;
pub mod players;

use crate::furuyoni_lib::net::message_sender::IntoMessageMap;
use furuyoni_lib::net::frames::*;
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::rules::PlayerPos;
use players::{IdlePlayer, RemotePlayer};

use networking::{post_office, ServerConnectionReader, ServerConnectionWriter};

use crate::game::{Game, GameResult};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let listener = TcpListener::bind("127.0.0.1:4255").await.unwrap();

    loop {
        println!("Ready To Get New Connection!");

        let (socket, addr) = listener.accept().await.unwrap();

        println!("New Connection Started: {addr}");

        tokio::spawn(async move {
            main_server_component(socket).await;
        });
    }
}

async fn main_server_component(socket: TcpStream) {
    // Game Action
    // TODO: Get Player List And Put Two Players in to Spawn Game
    spawn_game(socket).await;
}

async fn spawn_game(socket: TcpStream) {
    let (
        lobby_to_player_requester,
        lobby_to_player_responder,
        game_to_player_requester,
        game_to_player_responder,
        post_office_task,
    ) = spawn_post_office(socket);
    let p1 = RemotePlayer::new(game_to_player_requester);
    let p2 = IdlePlayer {};

    let (game, observable) = Game::create_game();

    let res = game.run(Box::new(p1), Box::new(p2)).await.expect("todo");
    let winner_str = match res {
        GameResult::Draw => "Draw",
        GameResult::Winner(winner) => match winner {
            PlayerPos::P1 => "P1",
            PlayerPos::P2 => "P2",
        },
    };
    println!("Game ended. Winner: {winner_str}");

    post_office_task.abort();
}

fn spawn_post_office(
    stream: TcpStream,
) -> (
    MessageChannel<LobbyToPlayerRequest, PlayerToLobbyResponse>,
    MessageChannel<LobbyToPlayerResponse, PlayerToLobbyRequest>,
    MessageChannel<GameToPlayerRequest, PlayerToGameResponse>,
    MessageChannel<GameToPlayerResponse, PlayerToGameRequest>,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ServerConnectionReader::new(read_half);
    let writer = ServerConnectionWriter::new(write_half);

    let (player_to_lobby_response_tx, player_to_lobby_response_rx) = mpsc::channel(20);
    let (player_to_lobby_request_tx, player_to_lobby_request_rx) = mpsc::channel(20);

    let (player_to_game_response_tx, player_to_game_response_rx) = mpsc::channel(20);
    let (player_to_game_request_tx, player_to_game_request_rx) = mpsc::channel(20);

    let (server_message_tx, server_message_rx) = mpsc::channel(20);

    let post_office_joinhandle = tokio::spawn(async {
        tokio::select!(
            res = post_office::receive_posts(reader, player_to_game_response_tx, player_to_game_request_tx, player_to_lobby_response_tx, player_to_lobby_request_tx) =>
                println ! ("receive_posts has ended with result: {:?}", res),
            () = post_office::handle_send_requests(server_message_rx, writer) =>
                println ! ("game_handle_send_request has ended."),
        );
    });

    let lobby_to_player_sender = server_message_tx.clone().with_map(|request| {
        ServerMessageFrame::LobbyMessage(LobbyToPlayerMessage::Request(request))
    });

    let lobby_to_player_requester =
        MessageChannel::new(lobby_to_player_sender, player_to_lobby_response_rx);

    let lobby_to_player_response_sender = server_message_tx.clone().with_map(|response| {
        ServerMessageFrame::LobbyMessage(LobbyToPlayerMessage::Response(response))
    });

    let lobby_to_player_responder =
        MessageChannel::new(lobby_to_player_response_sender, player_to_lobby_request_rx);

    let game_to_player_req_sender = server_message_tx
        .clone()
        .with_map(|request| ServerMessageFrame::GameMessage(GameToPlayerMessage::Request(request)));

    let game_to_player_requester =
        MessageChannel::new(game_to_player_req_sender, player_to_game_response_rx);

    let game_to_player_response_sender = server_message_tx.clone().with_map(|response_data| {
        ServerMessageFrame::GameMessage(GameToPlayerMessage::Response(response_data))
    });

    let game_to_player_responder =
        MessageChannel::new(game_to_player_response_sender, player_to_game_request_rx);

    return (
        lobby_to_player_requester,
        lobby_to_player_responder,
        game_to_player_requester,
        game_to_player_responder,
        post_office_joinhandle,
    );
}
