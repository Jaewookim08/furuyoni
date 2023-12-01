use crate::game::states::player_state::PlayerState;
use furuyoni_lib::rules::states::petals::Petals;
use furuyoni_lib::rules::states::PlayersData;
use furuyoni_lib::rules::PetalPosition;

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

    pub fn get_petals_mut(&mut self, petal_position: PetalPosition) -> &'_ mut Petals {
        match petal_position {
            PetalPosition::Distance => &mut self.distance,
            PetalPosition::Dust => &mut self.dust,
            PetalPosition::Aura(player) => &mut self.player_states[player].aura,
            PetalPosition::Flare(player) => &mut self.player_states[player].flare,
            PetalPosition::Life(player) => &mut self.player_states[player].life,
        }
    }
}
