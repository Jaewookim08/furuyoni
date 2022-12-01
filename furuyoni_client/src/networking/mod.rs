mod game_message_receiver;
mod message_manager;

use furuyoni_lib::net::connection::Connection;

pub use {
    game_message_receiver::GameMessageHandler, message_manager::GameConnection,
    message_manager::MessageManager,
};
