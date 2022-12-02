use furuyoni_lib::net::connection::Connection;
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub type GameConnection = Connection<ServerMessageFrame, ClientMessageFrame>;
