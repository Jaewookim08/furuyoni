use crate::game::states::player_state::PlayerState;
use furuyoni_lib::rules::states::petals::Petals;
use furuyoni_lib::rules::states::PlayersData;
use furuyoni_lib::rules::PetalsPosition;

pub(crate) struct BoardState {
    pub distance: Petals,
    pub dust: Petals,
    pub player_states: PlayerStates,
}
pub(crate) type PlayerStates = PlayersData<PlayerState>;

impl BoardState {
    pub fn new(distance: Petals, dust: Petals, player_states: PlayerStates) -> Self {
        Self {
            distance,
            dust,
            player_states,
        }
    }

    pub fn get_petals_mut(&mut self, petal_position: PetalsPosition) -> &'_ mut Petals {
        match petal_position {
            PetalsPosition::Distance => &mut self.distance,
            PetalsPosition::Dust => &mut self.dust,
            PetalsPosition::Aura(player) => &mut self.player_states[player].aura,
            PetalsPosition::Flare(player) => &mut self.player_states[player].flare,
            PetalsPosition::Life(player) => &mut self.player_states[player].life,
        }
    }
}
