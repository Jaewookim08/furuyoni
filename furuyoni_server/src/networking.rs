pub mod post_office;

use furuyoni_lib::net::connection::{ConnectionReader, ConnectionWriter};
use furuyoni_lib::net::frames::{ClientMessageFrame, ServerMessageFrame};

pub type ServerConnectionReader<TRead> = ConnectionReader<TRead, ClientMessageFrame>;
pub type ServerConnectionWriter<TWrite> = ConnectionWriter<TWrite, ServerMessageFrame>;
