use furuyoni_lib::net::connection;
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub mod post_office;

pub type ClientConnectionReader<TRead> = connection::ConnectionReader<TRead, ServerMessageFrame>;
pub type ClientConnectionWriter<TWrite> = connection::ConnectionWriter<TWrite, ClientMessageFrame>;
