mod cli_player;
mod idle_player;
mod player;
mod remote_player;

pub(crate) use {
    cli_player::CliPlayer, idle_player::IdlePlayer, player::Player, remote_player::RemotePlayer,
};
