use serde::{Serialize, Deserialize};

use super::base::WithRequestId;

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerMessage {
    Request(LobbyToPlayerRequest),
    Response(LobbyToPlayerResponseFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerRequest {
    Notify(LobbyToPlayerNotification),
    RequestData(LobbyToPlayerRequestDataFrame),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerNotification {}
pub type LobbyToPlayerRequestDataFrame = WithRequestId<LobbyToPlayerRequestData>;

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerRequestData {
    AreYouAlive
}

pub type LobbyToPlayerResponseFrame = WithRequestId<LobbyToPlayerResponse>;

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerResponse {
    RoomsList(Vec<LobbyRoomInfo>),
    RoonEnterSuccess(bool)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyRoomInfo{
    roon_id: u32,
    room_name: String,
    room_description: String,
    current_player_num: u32,
    max_player_num: u32,
}



#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToLobbyMessage{
    Response(PlayerToLobbyResponseFrame),
    Request(PlayerToLobbyRequestFrame)
}

pub type PlayerToLobbyResponseFrame = WithRequestId<PlayerToLobbyResponse>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToLobbyResponse {
    IAmAlive
 }

pub type PlayerToLobbyRequestFrame = WithRequestId<PlayerToLobbyRequest>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToLobbyRequest {
    GetRoomsList,
    TryEnterRoom(PlayerToLobbyTryEnterRoom),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerToLobbyTryEnterRoom{
    pub room_id: u32,
    pub player_id: u32
}