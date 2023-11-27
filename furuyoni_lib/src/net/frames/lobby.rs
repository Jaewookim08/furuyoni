use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerMessage {
    Request(LobbyToPlayerRequest),
    Response(LobbyToPlayerResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerRequest {
    Notify(LobbyToPlayerNotification),
    RequestData(LobbyToPlayerRequestData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerNotification {}

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyToPlayerRequestData {
    AreYouAlive
}

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
    Response(PlayerToLobbyResponse),
    Request(PlayerToLobbyRequest)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerToLobbyResponse {
    Ack
 }

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