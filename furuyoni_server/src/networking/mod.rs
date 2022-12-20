pub mod post_office;

mod message_sender;
pub use {message_sender::MessageSender, message_sender::SendError as MessageSendError};

mod message_receiver;
pub use {message_receiver::MessageReceiver, message_receiver::RecvError as MessageRecvError};

mod game_to_player_connection;

pub use {
    game_to_player_connection::Error as GameCommunicationError,
    game_to_player_connection::GameToPlayerConnection,
};

use furuyoni_lib::net::connection::{ConnectionReader, ConnectionWriter};
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub type ServerConnectionReader<TRead> = ConnectionReader<TRead, ClientMessageFrame>;
pub type ServerConnectionWriter<TWrite> = ConnectionWriter<TWrite, ServerMessageFrame>;
