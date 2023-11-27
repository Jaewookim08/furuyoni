use bevy::prelude::*;
use furuyoni_lib::player_actions::BasicAction;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct PickerPlugin;

#[derive(Event)]
pub struct RequestPick {
    basic_actions: HashSet<BasicAction>,
    skip: bool,
}
impl RequestPick {
    pub fn new(basic_actions: HashSet<BasicAction>, skip: bool) -> Self {
        Self {
            basic_actions,
            skip,
        }
    }
}

#[derive(Event, Reflect, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect_value(Serialize, Deserialize)]
pub enum PickedEvent {
    BasicAction(BasicAction),
    Skip,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SkipButton;

#[derive(Component, Reflect, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[reflect_value(Serialize, Deserialize, Component)]
pub struct BasicActionButton {
    pub action: BasicAction,
}

impl Default for BasicActionButton {
    fn default() -> Self {
        Self {
            action: BasicAction::MoveForward,
        }
    }
}

impl Plugin for PickerPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_state::<PickingState>()
            .register_type::<SkipButton>()
            .register_type::<BasicActionButton>()
            .add_event::<RequestPick>()
            .add_event::<PickedEvent>();
        // .add_systems(OnEnter(PickingState::Idle), disable_picker_buttons)
        // .add_systems(
        //     Update,
        //     (
        //         start_picker_on_request.run_if(in_state(PickingState::Idle)),
        //         poll_pickers.run_if(in_state(PickingState::Picking)),
        //     ),
        // );
    }
}
//
// #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
// enum PickingState {
//     #[default]
//     Idle,
//     Picking,
// }
//
// fn disable_picker_buttons(
//     mut buttons: Query<&mut Visibility, Or<(With<SkipButton>, With<BasicActionButton>)>>,
// ) {
//     for mut v in buttons.iter_mut() {
//         *v = Visibility::Hidden;
//     }
// }
//
// fn start_picker_on_request(
//     mut request: EventReader<RequestPick>,
//     mut set: ParamSet<(
//         Query<&mut Visibility, With<SkipButton>>,
//         Query<(&BasicActionButton, &mut Visibility)>,
//     )>,
//     mut next_state: ResMut<NextState<PickingState>>,
// ) {
//     if let Some(req) = request.iter().next() {
//         if req.skip {
//             for mut v in set.p0().iter_mut() {
//                 *v = Visibility::Inherited;
//             }
//         }
//
//         for (ba, mut v) in set.p1().iter_mut() {
//             if req.basic_actions.contains(&ba.action) {
//                 *v = Visibility::Inherited;
//             }
//         }
//         next_state.set(PickingState::Picking);
//     }
// }
//
// fn poll_pickers(
//     mut ev_picked: EventWriter<PickedEvent>,
//     basic_action_buttons: Query<(&Interaction, &BasicActionButton), Changed<Interaction>>,
//     skip_buttons: Query<&Interaction, (Changed<Interaction>, With<SkipButton>)>,
//     mut next_state: ResMut<NextState<PickingState>>,
// ) {
//     let picked = 'picked: {
//         for (interaction, ba) in basic_action_buttons.iter() {
//             match interaction {
//                 Interaction::Pressed => {
//                     break 'picked Some(PickedEvent::BasicAction(ba.action));
//                 }
//                 _ => {}
//             }
//         }
//
//         for interaction in skip_buttons.iter() {
//             match interaction {
//                 Interaction::Clicked => break 'picked Some(PickedEvent::Skip),
//                 _ => (),
//             }
//         }
//
//         None
//     };
//
//     if let Some(picked) = picked {
//         ev_picked.send(picked);
//         next_state.set(PickingState::Idle);
//     }
// }
