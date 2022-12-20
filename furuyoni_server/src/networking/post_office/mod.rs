use crate::networking::{ServerConnectionReader, ServerConnectionWriter};
use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{
    ClientMessageFrame, GameMessageFrame, GameNotification, GameRequest, GameRequestFrame,
    PlayerResponse, PlayerResponseFrame, ServerMessageFrame,
};
use furuyoni_lib::net::with_send_callback::WithSendCallback;
use rand::Rng;
use std::sync::atomic::AtomicU32;
use std::sync::MutexGuard;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot, Mutex};

pub async fn receive_posts<T: AsyncRead + Unpin>(
    mut reader: ServerConnectionReader<T>,
    player_message_tx: mpsc::Sender<PlayerResponseFrame>,
) {
    loop {
        match reader.read_frame().await {
            Err(err) => {
                println!("Error occurred while reading a frame. Err: {:?}", err);
                return;
            }
            Ok(client_message_frame) => match client_message_frame {
                ClientMessageFrame::PlayerResponse(response) => {
                    player_message_tx.send(response);
                }
            },
        }
    }
}

pub async fn handle_send_requests<TWrite: AsyncWrite + Unpin + Send>(
    mut send_game_message_mailbox: mpsc::Receiver<WithSendCallback<GameMessageFrame>>,
    mut writer: ServerConnectionWriter<TWrite>,
) {
    while let Some(request) = send_game_message_mailbox.recv().await {
        let res = writer
            .write_frame(&ServerMessageFrame::GameMessage(request.data))
            .await;

        let _ = request.callback.send(res.map_err(|e| e.into()));
    }

    println!("[PostOffice] No more messages to send. 'handle_send_requests' has ended.")
}
