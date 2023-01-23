extern crate furuyoni_lib;

use crate::networking::{ServerConnectionReader, ServerConnectionWriter};
use std::sync::Arc;

use furuyoni_lib::net::frames::{
    GameMessageFrame, GameRequest, GameToPlayerRequestData, GameToPlayerRequestDataFrame,
    PlayerResponse, PlayerResponseFrame, RequestMainPhaseAction, WriteError,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::{IntoMessageMap, MessageSender};
use furuyoni_lib::net::{MessageReceiver, RequestError, Requester};
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

    let (game_to_player_requester, post_office_task) = spawn_post_office(socket);
    let p1 = RemotePlayer::new(game_to_player_requester);

    let mut game = game::Game::new(Box::new(p1), Box::new(IdlePlayer {}));
    let res = futures::executor::block_on(game.run());
    let winner_str = match res.winner {
        PlayerPos::P1 => "P1",
        PlayerPos::P2 => "P2",
    };
    println!("Game ended. Winner: {winner_str}");

    post_office_task.abort();
}

fn spawn_post_office<'a>(
    stream: TcpStream,
) -> (
    impl Requester<GameToPlayerRequestData, Response = PlayerResponse>,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ServerConnectionReader::new(read_half);
    let writer = ServerConnectionWriter::new(write_half);

    let (player_response_tx, player_response_rx) = tokio::sync::mpsc::channel(20);
    let (player_request_tx, player_request_rx) = tokio::sync::mpsc::channel(20);
    let (game_message_tx, game_message_rx) = tokio::sync::mpsc::channel(20);

    let post_office_joinhandle = tokio::spawn(async {
        let (_, _) = tokio::join!(
            post_office::receive_posts(reader, player_response_tx, player_request_tx),
            post_office::handle_send_requests(game_message_rx, writer),
        );
        ()
    });

    let game_message_sender = Arc::new(game_message_tx);

    let game_to_player_req_sender = game_message_sender
        .clone()
        .with_map(|request_data| GameMessageFrame::Request(GameRequest::RequestData(request_data)));

    let game_to_player_requester =
        MessageChannel::new(game_to_player_req_sender, player_response_rx);

    return (game_to_player_requester, post_office_joinhandle);
}
