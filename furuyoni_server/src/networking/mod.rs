pub mod post_office;

mod game_to_player_connection;

pub use game_to_player_connection::GameToPlayerRequester;

use furuyoni_lib::net::connection::{ConnectionReader, ConnectionWriter};
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub type ServerConnectionReader<TRead> = ConnectionReader<TRead, ClientMessageFrame>;
pub type ServerConnectionWriter<TWrite> = ConnectionWriter<TWrite, ServerMessageFrame>;
