extern crate furuyoni_lib;

use furuyoni_lib::net::{frames::{
    LobbyToPlayerRequestData, LobbyResponse,
    GameNotification, GameToPlayerRequestData, PlayerResponse, GameToPlayerResponseFrame, LobbyNotification, PlayerToGameRequestFrame,
}, Requester, message_sender::MessageSender, Responder};

struct MainChannels{
    lobby_to_player_requester : Box<dyn Requester<LobbyToPlayerRequestData, Response = LobbyResponse>>,
    lobby_to_player_notifier : Box<dyn MessageSender<LobbyNotification>>,
    lobby_to_player_responder : Box<dyn Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame>>,
    game_to_player_requester : Box<dyn Requester<GameToPlayerRequestData, Response = PlayerResponse>>,
    game_to_player_notifier : Box<dyn MessageSender<GameNotification>>,
    game_to_player_responder : Box<dyn Responder<GameToPlayerResponseFrame, Request = PlayerToGameRequestFrame>>
}

