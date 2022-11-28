use furuyoni_lib::net::connection::Connection;
use furuyoni_lib::net::frames::{GameMessageFrame, PlayerMessageFrame};

pub type GameConnection = Connection<GameMessageFrame, PlayerMessageFrame>;
