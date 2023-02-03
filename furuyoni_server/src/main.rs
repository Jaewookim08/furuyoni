extern crate furuyoni_lib;

use crate::networking::{ServerConnectionReader, ServerConnectionWriter};
use std::sync::Arc;

use furuyoni_lib::net::frames::{
    GameMessageFrame, GameNotification, GameRequest, GameToPlayerRequestData,
    GameToPlayerRequestDataFrame, GameToPlayerResponseFrame, PlayerResponse, PlayerResponseFrame,
    PlayerToGameRequestFrame, RequestMainPhaseAction, WriteError,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::{IntoMessageMap, MessageSender};
use furuyoni_lib::net::{MessageReceiver, RequestError, Requester, Responder};
use furuyoni_lib::players::{CliPlayer, IdlePlayer};
use furuyoni_lib::rules::PlayerPos;
use networking::post_office;
use remote_player::RemotePlayer;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

mod game;
mod networking;
mod remote_player;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:4255").await.unwrap();
    let (socket, _) = listener.accept().await.unwrap();

    let (
        game_to_player_requester,
        game_to_player_notifier,
        game_to_player_responder,
        post_office_task,
    ) = spawn_post_office(socket);
    let p1 = RemotePlayer::new(game_to_player_requester, game_to_player_notifier);
    let p2 = CliPlayer {};

    let mut game = game::Game::new(Box::new(p1), Box::new(p2));

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
    impl Requester<GameToPlayerRequestData, Response = PlayerResponse>,
    impl MessageSender<GameNotification>,
    impl Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame>,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ServerConnectionReader::new(read_half);
    let writer = ServerConnectionWriter::new(write_half);

    let (player_response_tx, player_response_rx) = tokio::sync::mpsc::channel(20);
    let (player_request_tx, player_request_rx) = tokio::sync::mpsc::channel(20);
    let (game_message_tx, game_message_rx) = tokio::sync::mpsc::channel(20);

    let post_office_joinhandle = tokio::spawn(async {
        tokio::select!(
            res = post_office::receive_posts(reader, player_response_tx, player_request_tx) =>
                println!("receive_posts has ended with result: {:?}", res),
            () = post_office::handle_send_requests(game_message_rx, writer) =>
                println!("handle_send_request has ended."),
        );
    });

    let game_to_player_req_sender = game_message_tx
        .clone()
        .with_map(|request_data| GameMessageFrame::Request(GameRequest::RequestData(request_data)));

    let game_to_player_requester =
        MessageChannel::new(game_to_player_req_sender, player_response_rx);

    let game_to_player_response_sender =
        game_message_tx.clone().with_map(GameMessageFrame::Response);

    let game_to_player_responder =
        MessageChannel::new(game_to_player_response_sender, player_request_rx);

    let game_to_player_notifier = game_message_tx
        .clone()
        .with_map(|m| GameMessageFrame::Request(GameRequest::Notify(m)));

    return (
        game_to_player_requester,
        game_to_player_notifier,
        game_to_player_responder,
        post_office_joinhandle,
    );
}
