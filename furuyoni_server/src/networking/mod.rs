use furuyoni_lib::net::connection::{ConnectionReader, ConnectionWriter};
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub type GameConnectionReader = ConnectionReader<ClientMessageFrame>;
pub type GameConnectionWriter = ConnectionWriter<ServerMessageFrame>;
