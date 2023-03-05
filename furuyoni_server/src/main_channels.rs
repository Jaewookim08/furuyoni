extern crate furuyoni_lib;

use std::error::Error;

use furuyoni_lib::net::{frames::{
    LobbyToPlayerRequestData, LobbyResponse,
    GameNotification, GameToPlayerRequestData, PlayerResponse, GameToPlayerResponseFrame, LobbyNotification, PlayerToGameRequestFrame,
}, Requester, message_sender::MessageSender, Responder, connection::ParseError};

pub struct MainChannels{
    lobby_to_player_requester : Box<dyn Requester<LobbyToPlayerRequestData, Response = LobbyResponse, Error = ParseError>>,
    lobby_to_player_notifier : Box<dyn MessageSender<LobbyNotification>>,
    lobby_to_player_responder : Box<dyn Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame, Error = ParseError>>,
    game_to_player_requester : Box<dyn Requester<GameToPlayerRequestData, Response = PlayerResponse, Error = ParseError>>,
    game_to_player_notifier : Box<dyn MessageSender<GameNotification>>,
    game_to_player_responder : Box<dyn Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame, Error = ParseError>>
}

