use bevy::prelude::*;
use furuyoni_lib::player_actions::BasicAction;
use iyes_loopless::prelude::*;
use std::collections::HashSet;

pub struct PickerPlugin;

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

pub enum PickedEvent {
    BasicAction(BasicAction),
    Skip,
}

#[derive(Component)]
pub struct SkipButton;

#[derive(Component)]
pub struct BasicActionButton {
    pub action: BasicAction,
}

impl Plugin for PickerPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(PickingState::Idle)
            .add_event::<RequestPick>()
            .add_event::<PickedEvent>()
            .add_enter_system(PickingState::Idle, disable_picker_buttons)
            .add_system(start_request_on_request.run_in_state(PickingState::Idle))
            .add_system(poll_pickers.run_in_state(PickingState::Picking));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PickingState {
    Idle,
    Picking,
}

fn disable_picker_buttons(
    mut buttons: Query<(&mut Visibility), Or<(With<SkipButton>, With<BasicActionButton>)>>,
) {
    for mut v in buttons.iter_mut() {
        v.is_visible = false;
    }
}

fn start_request_on_request(
    mut commands: Commands,
    mut request: EventReader<RequestPick>,
    mut set: ParamSet<(
        Query<&mut Visibility, With<SkipButton>>,
        Query<(&BasicActionButton, &mut Visibility)>,
    )>,
) {
    if let Some(req) = request.iter().next() {
        if req.skip {
            for mut v in set.p0().iter_mut() {
                v.is_visible = true;
            }
        }

        for (ba, mut v) in set.p1().iter_mut() {
            if req.basic_actions.contains(&ba.action) {
                v.is_visible = true;
            }
        }
    }
    commands.insert_resource(NextState(PickingState::Picking));
}

fn poll_pickers(
    mut ev_picked: EventWriter<PickedEvent>,
    mut commands: Commands,
    mut basic_action_buttons: Query<(&Interaction, &BasicActionButton), Changed<Interaction>>,
    mut skip_buttons: Query<&Interaction, (Changed<Interaction>, With<SkipButton>)>,
) {
    let picked = 'picked: {
        for (interaction, ba) in basic_action_buttons.iter() {
            match interaction {
                Interaction::Clicked => {
                    break 'picked Some(PickedEvent::BasicAction(ba.action));
                }
                _ => {}
            }
        }

        for interaction in skip_buttons.iter() {
            match interaction {
                Interaction::Clicked => break 'picked Some(PickedEvent::Skip),
                _ => (),
            }
        }

        None
    };

    if let Some(picked) = picked {
        ev_picked.send(picked);
        commands.insert_resource(NextState(PickingState::Idle));
    }
}
