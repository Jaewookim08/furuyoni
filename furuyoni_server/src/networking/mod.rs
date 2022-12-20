pub mod post_office;

pub mod game_player_connection;

mod game_communication_mananger;
pub use {
    game_communication_mananger::Error as GameCommunicationError,
    game_communication_mananger::GameCommunicationManager,
};

use furuyoni_lib::net::connection::{ConnectionReader, ConnectionWriter};
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub type ServerConnectionReader<TRead> = ConnectionReader<TRead, ClientMessageFrame>;
pub type ServerConnectionWriter<TWrite> = ConnectionWriter<TWrite, ServerMessageFrame>;
