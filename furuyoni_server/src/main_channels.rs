extern crate furuyoni_lib;

use furuyoni_lib::net::frames::{GameToPlayerRequest, LobbyToPlayerRequest, PlayerToGameResponse, PlayerToLobbyResponse};
use furuyoni_lib::net::message_channel::MessageChannel;

pub struct MainChannels {
    lobby_to_player_channel: MessageChannel<LobbyToPlayerRequest, PlayerToLobbyResponse>,
    game_to_player_channel: MessageChannel<GameToPlayerRequest, PlayerToGameResponse>,
}

