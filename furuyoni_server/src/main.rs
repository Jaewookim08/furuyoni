extern crate furuyoni_lib;
use furuyoni_lib::net::frames::*;
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::{IntoMessageMap, MessageSender};
use furuyoni_lib::net::{Requester, Responder};
use furuyoni_lib::players::{IdlePlayer};
use furuyoni_lib::rules::PlayerPos;

mod game;
use crate::game::Game;

mod networking;
use networking::{post_office, ServerConnectionReader, ServerConnectionWriter};

mod remote_player;
use remote_player::RemotePlayer;

mod main_channels;
use main_channels::MainChannels;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let listener = TcpListener::bind("127.0.0.1:4255").await.unwrap();

    loop{
        println!("Ready To Get New Connection!");

        let (socket, addr) = listener.accept().await.unwrap();

        println!("New Connection Started: {addr}");

        tokio::spawn(async move {
            main_server_component(socket).await;
        });
    }
}

async fn main_server_component(socket: TcpStream){
    // Game Action
    // TODO: Get Player List And Put Two Players in to Spawn Game
    spawn_game(socket).await;
}

async fn spawn_game(socket: TcpStream){
    let (
        lobby_to_player_requester,
        lobby_to_player_notifier,
        lobby_to_player_responder,
        game_to_player_requester,
        game_to_player_notifier,
        game_to_player_responder,
        post_office_task,
    ) = spawn_post_office(socket);
    let p1 = RemotePlayer::new(game_to_player_requester, game_to_player_notifier);
    let p2 = IdlePlayer {};

    let mut game = Game::new(Box::new(p1), Box::new(p2));

    let res = game.run().await;
    let winner_str = match res.winner {
        PlayerPos::P1 => "P1",
        PlayerPos::P2 => "P2",
    };
    println!("Game ended. Winner: {winner_str}");

    post_office_task.abort();
}

fn spawn_post_office(
    stream: TcpStream,
) -> (
    impl Requester<LobbyToPlayerRequestData, Response = PlayerToLobbyResponse>,
    impl MessageSender<LobbyToPlayerNotification>,
    impl Responder<LobbyToPlayerResponseFrame, Request = PlayerToLobbyRequestFrame>,
    impl Requester<GameToPlayerRequestData, Response = PlayerToGameResponse>,
    impl MessageSender<GameToPlayerNotification>,
    impl Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame>,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ServerConnectionReader::new(read_half);
    let writer = ServerConnectionWriter::new(write_half);

    let (lobby_player_response_tx, lobby_player_response_rx) = mpsc::channel(20);
    let (lobby_player_request_tx, lobby_player_request_rx) = mpsc::channel(20);
    let (lobby_message_tx, lobby_message_rx) = mpsc::channel(20);
    
    let (game_player_response_tx, game_player_response_rx) = mpsc::channel(20);
    let (game_player_request_tx, game_player_request_rx) = mpsc::channel(20);
    let (game_message_tx, game_message_rx) = mpsc::channel(20);
    
    let post_office_joinhandle = tokio::spawn(async {
        tokio::select!(
            res = post_office::receive_posts(reader, game_player_response_tx, game_player_request_tx, lobby_player_response_tx, lobby_player_request_tx) =>
                println!("receive_posts has ended with result: {:?}", res),
            () = post_office::handle_send_requests(game_message_rx, writer) =>
                println!("game_handle_send_request has ended."),
        );
    });

    let lobby_to_player_req_sender = lobby_message_tx
        .clone()
        .with_map(|request_data| LobbyToPlayerMessage::Request(LobbyToPlayerRequest::RequestData(request_data)));

    let lobby_to_player_requester =
        MessageChannel::new(lobby_to_player_req_sender, lobby_player_response_rx);

    let lobby_to_player_response_sender =
        lobby_message_tx.clone().with_map(LobbyToPlayerMessage::Response);

    let lobby_to_player_responder =
        MessageChannel::new(lobby_to_player_response_sender, lobby_player_request_rx);

    let lobby_to_player_notifier = lobby_message_tx
        .clone()
        .with_map(|m| LobbyToPlayerMessage::Request(LobbyToPlayerRequest::Notify(m)));

    let game_to_player_req_sender = game_message_tx
        .clone()
        .with_map(|request_data| GameToPlayerMessage::Request(GameToPlayerRequest::RequestData(request_data)));

    let game_to_player_requester =
        MessageChannel::new(game_to_player_req_sender, game_player_response_rx);

    let game_to_player_response_sender =
        game_message_tx.clone().with_map(GameToPlayerMessage::Response);

    let game_to_player_responder =
        MessageChannel::new(game_to_player_response_sender, game_player_request_rx);

    let game_to_player_notifier = game_message_tx
        .clone()
        .with_map(|m| GameToPlayerMessage::Request(GameToPlayerRequest::Notify(m)));

    return (
        lobby_to_player_requester,
        lobby_to_player_notifier,
        lobby_to_player_responder,
        game_to_player_requester,
        game_to_player_notifier,
        game_to_player_responder,
        post_office_joinhandle,
    );
}
