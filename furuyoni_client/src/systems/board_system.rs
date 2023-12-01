use crate::ReflectComponent;
use bevy::prelude::Component;
use bevy::reflect::Reflect;
use furuyoni_lib::rules::states::{ViewablePlayerState, ViewableState};
use furuyoni_lib::rules::PlayerPos;

// use crate::systems::picker::{PickedEvent, RequestPick};
// use bevy::prelude::*;
// use bevy::text::Text;
// use furuyoni_lib::events::{EventCost, GameEvent};
// use furuyoni_lib::net::frames::RequestMainPhaseAction;
// use furuyoni_lib::player_actions::{
//     BasicAction, BasicActionCost, MainPhaseAction, PlayBasicAction,
// };
// use furuyoni_lib::rules::{PlayerPos, ViewablePlayerState, ViewableState};
// use std::collections::VecDeque;
// use tokio::sync::oneshot;
//
// use crate::game_logic::SelfPlayerPos;
//
// pub struct BoardPlugin;
//
// impl Plugin for BoardPlugin {
//     fn build(&self, app: &mut App) {
//         app.insert_resource(BoardState::Idle)
//             .init_resource::<BoardRequestQueue>()
//             .register_type::<StateLabel>()
//             .register_type::<StateStringPicker>()
//             .register_type::<PlayerValuePicker>()
//             .register_type::<PlayerRelativePos>()
//             .register_type::<PlayerValuePickerType>()
//             .add_systems(
//                 Update,
//                 display_board
//                     .run_if(resource_exists::<BoardState>())
//                     .run_if(resource_exists::<SelfPlayerPos>()),
//             );
//         // .add_system(run_system.run_if(resource_exists::<BoardState>()));
//     }
// }
//
// #[derive(Debug)]
// pub enum BoardRequest {
//     GetMainPhaseAction {
//         query: RequestMainPhaseAction,
//         callback: oneshot::Sender<MainPhaseAction>,
//     },
//     PlayEvent(GameEvent),
// }
//
// #[derive(Debug, Default, Resource)]
// pub struct BoardRequestQueue(pub VecDeque<BoardRequest>);
//
// fn display_board(
//     state: Res<BoardState>,
//     self_pos: Res<SelfPlayerPos>,
//     mut query: Query<(&mut Text, &StateLabel)>,
// ) {
//     if state.is_changed() {
//         for (mut text, state_label) in &mut query {
//             text.sections[state_label.text_section_index].value =
//                 get_string(self_pos.0, state.0, &state_label.picker);
//         }
//     }
// }
//
#[derive(Debug, Reflect, Default)]
pub enum PlayerRelativePos {
    #[default]
    Me,
    Opponent,
}

#[derive(Debug, Reflect, Default)]
pub struct PlayerValuePicker {
    pos: PlayerRelativePos,
    value_type: PlayerValuePickerType,
}

impl PlayerValuePicker {
    pub fn new(pos: PlayerRelativePos, value_type: PlayerValuePickerType) -> Self {
        Self { pos, value_type }
    }
}

// Todo: refactor using PetalsPos.
#[derive(Debug, Reflect, Default)]
pub enum StateStringPicker {
    #[default]
    Dust,
    Distance,
    Turn,
    PlayerValue(PlayerValuePicker),
}

// Todo: remove. We have PetalsPos.
#[derive(Debug, Reflect, Default)]
pub enum PlayerValuePickerType {
    #[default]
    Life,
    Flare,
    Aura,
    Vigor,
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct StateLabel {
    text_section_index: usize,
    picker: StateStringPicker,
}

impl StateLabel {
    pub fn new(text_section_index: usize, picker: StateStringPicker) -> Self {
        Self {
            text_section_index,
            picker,
        }
    }
}
//
// #[derive(Resource)]
// enum BoardState {
//     Idle,
//     PlayingGameEvent,
//     WaitForMainPhaseAction {
//         callback: Option<oneshot::Sender<MainPhaseAction>>,
//     },
// }
//
// fn board_is_idle(board_state: Res<BoardState>) -> bool {
//     match board_state.into_inner() {
//         BoardState::Idle => true,
//         _ => false,
//     }
// }
//
// fn get_string(self_pos: PlayerPos, state: &ViewableState, picker: &StateStringPicker) -> String {
//     match picker {
//         StateStringPicker::Dust => state.dust.to_string(),
//         StateStringPicker::Distance => state.distance.to_string(),
//         StateStringPicker::Turn => state.turn_number.to_string(),
//         StateStringPicker::PlayerValue(PlayerValuePicker { pos, value_type }) => {
//             let pos = match pos {
//                 PlayerRelativePos::Me => self_pos,
//                 PlayerRelativePos::Opponent => self_pos.other(),
//             };
//             let player = &state.player_states[pos];
//
//             get_string_from_player(player, value_type)
//         }
//     }
// }
//
// fn get_string_from_player(
//     player_state: &ViewablePlayerState,
//     picker: &PlayerValuePickerType,
// ) -> String {
//     match picker {
//         PlayerValuePickerType::Life => player_state.get_life().to_string(),
//         PlayerValuePickerType::Flare => player_state.get_flare().to_string(),
//         PlayerValuePickerType::Aura => player_state.get_aura().to_string(),
//         PlayerValuePickerType::Vigor => player_state.get_vigor().to_string(),
//     }
// }
//
// fn apply_event(game_state: &mut BoardState, event: &GameEvent) {
//     let state = &mut game_state.0;
//     match event {
//         GameEvent::DoBasicAction {
//             pos,
//             cost,
//             action,
//             amount,
//         } => {
//             if let Some(cost) = cost {
//                 apply_cost(state, *pos, cost);
//             }
//
//             // Todo: petalsSlot과 transfer로 수정.. unsafe cast 수정.
//             let amount = *amount as u32;
//
//             match action {
//                 BasicAction::MoveForward => {
//                     *state.player_states[pos].get_aura_mut() += amount;
//                     state.distance -= amount;
//                 }
//                 BasicAction::MoveBackward => {
//                     *state.player_states[pos].get_aura_mut() -= amount;
//                     state.distance += amount;
//                 }
//                 BasicAction::Recover => {
//                     *state.player_states[pos].get_aura_mut() += amount;
//                     state.dust -= amount;
//                 }
//                 BasicAction::Focus => {
//                     *state.player_states[pos].get_aura_mut() -= amount;
//                     *state.player_states[pos].get_flare_mut() += amount;
//                 }
//             }
//         }
//     }
// }
//
// fn apply_cost(state: &mut ViewableState, pos: PlayerPos, cost: &EventCost) {
//     match cost {
//         EventCost::Vigor => {
//             *state.player_states[pos].get_vigor_mut() -= 1;
//         }
//     }
// }

// fn run_system(
//     board_state: ResMut<BoardState>,
//     request_queue: ResMut<BoardRequestQueue>,
//     pick_requester: EventWriter<RequestPick>,
//     picked_event_reader: EventReader<PickedEvent>,
//     game_state: ResMut<BoardState>,
// ) {
//     let board_state = board_state.into_inner();
//     let op_next_state = match board_state {
//         BoardState::Idle => process_requests(
//             request_queue.into_inner(),
//             pick_requester,
//             game_state.into_inner(),
//         ),
//         BoardState::PlayingGameEvent => {
//             todo!()
//         }
//         BoardState::WaitForMainPhaseAction { callback } => {
//             wait_for_main_phase_action_picker(picked_event_reader, callback)
//         }
//     };
//
//     if let Some(next_state) = op_next_state {
//         *board_state = next_state;
//     }
// }

// fn process_requests(
//     request_queue: &mut BoardRequestQueue,
//     mut pick_requester: EventWriter<RequestPick>,
//     game_state: &mut BoardState,
// ) -> Option<BoardState> {
//     // Using 'if' instead of 'while' to process at max 1 request per frame.
//     if let Some(req) = request_queue.0.pop_front() {
//         match req {
//             BoardRequest::GetMainPhaseAction { callback, query } => {
//                 pick_requester.send(RequestPick::new(
//                     query.performable_basic_actions.iter().cloned().collect(),
//                     true,
//                 ));
//
//                 Some(BoardState::WaitForMainPhaseAction {
//                     callback: Some(callback),
//                 })
//             }
//             BoardRequest::PlayEvent(ev) => {
//                 apply_event(game_state, &ev);
//                 None
//             }
//         }
//     } else {
//         None
//     }
// }

// fn wait_for_main_phase_action_picker(
//     mut event_reader: EventReader<PickedEvent>,
//     callback: &mut Option<oneshot::Sender<MainPhaseAction>>,
// ) -> Option<BoardState> {
//     if let Some(ev) = event_reader.iter().next() {
//         let action = match ev {
//             PickedEvent::BasicAction(ba) => MainPhaseAction::PlayBasicAction(PlayBasicAction {
//                 action: *ba,
//                 cost: BasicActionCost::Vigor,
//             }),
//             PickedEvent::Skip => MainPhaseAction::EndMainPhase,
//         };
//
//         match callback.take() {
//             None => {
//                 todo!()
//             }
//             Some(s) => s.send(action).expect("todo: send failed"),
//         }
//
//         Some(BoardState::Idle)
//     } else {
//         None
//     }
// }
