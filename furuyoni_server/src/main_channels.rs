extern crate furuyoni_lib;

use furuyoni_lib::net::{frames::{
    LobbyToPlayerRequestData, LobbyToPlayerResponse,
    GameToPlayerNotification, GameToPlayerRequestData, PlayerToGameResponse, GameToPlayerResponseFrame, LobbyToPlayerNotification, PlayerToGameRequestFrame,
}, Requester, message_sender::MessageSender, Responder, connection::ParseError};

pub struct MainChannels{
    lobby_to_player_requester : Box<dyn Requester<LobbyToPlayerRequestData, Response = LobbyToPlayerResponse, Error = ParseError>>,
    lobby_to_player_notifier : Box<dyn MessageSender<LobbyToPlayerNotification>>,
    lobby_to_player_responder : Box<dyn Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame, Error = ParseError>>,
    game_to_player_requester : Box<dyn Requester<GameToPlayerRequestData, Response = PlayerToGameResponse, Error = ParseError>>,
    game_to_player_notifier : Box<dyn MessageSender<GameToPlayerNotification>>,
    game_to_player_responder : Box<dyn Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame, Error = ParseError>>
}

