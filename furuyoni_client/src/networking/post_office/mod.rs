use crate::networking::{ClientConnectionReader, ClientConnectionWriter};
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameToPlayerMessage, GameToPlayerRequest, GameToPlayerResponse,
    LobbyToPlayerMessage, PlayerToGameMessage, PlayerToGameRequest, PlayerToGameResponse,
    ServerMessageFrame,
};
use furuyoni_lib::net::message_channel::MessageChannel;
use furuyoni_lib::net::message_sender::IntoMessageMap;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Error, Debug)]
pub enum ReceivePostsError {
    #[error("Failed to send a message through a given channel.")]
    ChannelSendError,
    #[error("Failed to read a frame.")]
    FrameReadError,
}

pub fn spawn_post_office(
    stream: TcpStream,
) -> (
    MessageChannel<PlayerToGameRequest, GameToPlayerResponse>,
    MessageChannel<PlayerToGameResponse, GameToPlayerRequest>,
    JoinHandle<()>,
) {
    let (read_half, write_half) = stream.into_split();

    let reader = ClientConnectionReader::new(read_half);
    let writer = ClientConnectionWriter::new(write_half);

    let (game_to_player_request_tx, game_to_player_request_rx) = mpsc::channel(20);
    let (game_to_player_response_tx, game_to_player_response_rx) = mpsc::channel(20);

    let (client_message_tx, client_message_rx) = mpsc::channel(20);

    let post_office_joinhandle = tokio::spawn(async {
        tokio::select!(
            res = tokio::spawn(receive_posts(reader, game_to_player_request_tx, game_to_player_response_tx)) =>
                println!("receive_posts has ended with result: {:?}", res),
            res = tokio::spawn(handle_send_requests(client_message_rx, writer)) =>
                println!("handle_send_request has ended with result: {:?}", res),
        );
    });

    let player_to_game_request_sender = client_message_tx.clone().with_map(|request| {
        ClientMessageFrame::PlayerToGameMessage(PlayerToGameMessage::Request(request))
    });

    let player_to_game_requester =
        MessageChannel::new(player_to_game_request_sender, game_to_player_response_rx);

    let player_to_game_response_sender = client_message_tx
        .clone()
        .with_map(|r| ClientMessageFrame::PlayerToGameMessage(PlayerToGameMessage::Response(r)));

    let player_to_game_responder =
        MessageChannel::new(player_to_game_response_sender, game_to_player_request_rx);

    return (
        player_to_game_requester,
        player_to_game_responder,
        post_office_joinhandle,
    );
}

async fn receive_posts<T: AsyncRead + Unpin>(
    mut reader: ClientConnectionReader<T>,
    game_request_tx: mpsc::Sender<GameToPlayerRequest>,
    game_response_tx: mpsc::Sender<GameToPlayerResponse>,
) -> Result<(), ReceivePostsError> {
    loop {
        match reader.read_frame().await {
            Err(_) => {
                // Todo: Ignore error when the error is from a parsing failure.
                return Err(ReceivePostsError::FrameReadError);
            }
            Ok(message_frame) => match message_frame {
                ServerMessageFrame::GameMessage(msg) => match msg {
                    GameToPlayerMessage::Request(req) => {
                        game_request_tx
                            .try_send(req)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                    GameToPlayerMessage::Response(resp) => {
                        game_response_tx
                            .try_send(resp)
                            .map_err(|_| ReceivePostsError::ChannelSendError)?;
                    }
                },
                ServerMessageFrame::LobbyMessage(msg) => match msg {
                    LobbyToPlayerMessage::Request(req) => {
                        todo!()
                    }
                    LobbyToPlayerMessage::Response(res) => {
                        todo!()
                    }
                },
            },
        }
    }
}

async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut mailbox: mpsc::Receiver<ClientMessageFrame>,
    mut writer: ClientConnectionWriter<TWrite>,
) {
    while let Some(request) = mailbox.recv().await {
        let res = writer.write_frame(&request).await;

        if let Err(_) = res {
            panic!("Todo");
        }
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
