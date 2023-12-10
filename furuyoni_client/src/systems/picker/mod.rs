use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_tokio_tasks::TaskContext;
use furuyoni_lib::rules::player_actions::{BasicAction, BasicActionCost, MainPhaseAction};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct PickerPlugin;

pub enum PickMainPhaseActionResult {
    PayBasicActionCost(BasicActionCost),
    EndMainPhase,
}

pub async fn pick_main_phase_action(
    mut ctx: TaskContext,
    allowed_costs: Arc<Vec<BasicActionCost>>,
) -> PickMainPhaseActionResult {
    loop {
        let allowed_costs = allowed_costs.clone();
        let picked = pick_anything(&mut ctx, move |p| match p {
            Pickable::EndMainPhase => true,
            Pickable::Vigor => allowed_costs.contains(&BasicActionCost::Vigor),
            _ => false,
        })
        .await;

        match picked {
            Pickable::EndMainPhase => return PickMainPhaseActionResult::EndMainPhase,
            Pickable::Vigor => {
                return PickMainPhaseActionResult::PayBasicActionCost(BasicActionCost::Vigor)
            }
            _ => { /*retry */ }
        }
    }
}

pub enum PickBasicActionResult {
    BasicAction(BasicAction),
    Cancel,
}

pub async fn pick_basic_action(
    ctx: &TaskContext,
    allowed_basic_actions: Arc<Vec<BasicAction>>,
) -> PickBasicActionResult {
    loop {
        let allowed_basic_actions = allowed_basic_actions.clone();
        let picked = pick_anything(&ctx, move |p| match p {
            Pickable::Cancel => true,
            Pickable::BasicAction(b) => allowed_basic_actions.contains(&b),
            _ => false,
        })
        .await;

        match picked {
            Pickable::Cancel => return PickBasicActionResult::Cancel,
            Pickable::BasicAction(action) => return PickBasicActionResult::BasicAction(action),
            _ => { /*retry */ }
        }
    }
}

#[derive(Resource)]
struct PickerCallBack {
    sender: Option<oneshot::Sender<Pickable>>,
}

impl Plugin for PickerPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_state::<PickingState>()
            .register_type::<Pickable>()
            .register_type::<PickerButton>()
            .add_systems(
                Update,
                (poll_pickers.run_if(resource_exists::<PickerCallBack>()),),
            );
    }
}

#[derive(Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct PickerButton {
    pub pickable: Pickable,
}

#[derive(Default, Debug, Reflect, Serialize, Deserialize, Copy, Clone)]
#[reflect_value(Serialize, Deserialize)]
pub enum Pickable {
    #[default]
    Cancel,
    EndMainPhase,
    BasicAction(BasicAction),
    Vigor,
}

async fn pick_anything(
    ctx: &TaskContext,
    predicate: impl Fn(Pickable) -> bool + Send + Sync + 'static,
) -> Pickable {
    let (tx, rx) = oneshot::channel();

    ctx.dispatch_to_main_thread(move |ctx| {
        ctx.world
            .insert_resource(PickerCallBack { sender: Some(tx) });
        ctx.world.run_system_once(enable_pickers_with(predicate));
    });

    let ret = rx.await.expect(
        "todo: Picker system failed. Maybe the current request have been cancelled by another one.",
    );

    ctx.dispatch_to_main_thread(move |ctx| {
        ctx.world.run_system_once(disable_picker_buttons);
    });

    ret
}

fn enable_pickers_with(
    predicate: impl Fn(Pickable) -> bool + Send + Sync,
) -> impl Fn(Query<(&PickerButton, &mut Visibility)>) + Send + Sync {
    move |mut query: Query<(&PickerButton, &mut Visibility)>| {
        for (picker_button, mut visibility) in query.iter_mut() {
            *visibility = if predicate(picker_button.pickable) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn poll_pickers(
    mut callback: ResMut<PickerCallBack>,
    basic_action_buttons: Query<(&Interaction, &PickerButton), Changed<Interaction>>,
) {
    if callback.sender.is_none() {
        return;
    }

    let picked = 'picked: {
        for (interaction, picker) in basic_action_buttons.iter() {
            match interaction {
                // Todo: bevy ui update되면 맞춰서 수정. 기왕이면 released에.
                Interaction::Pressed => {
                    break 'picked Some(&picker.pickable);
                }
                _ => {}
            }
        }

        None
    };

    if let Some(picked) = picked {
        let sender = callback.sender.take().unwrap();
        sender.send(picked.clone()).expect("todo");
    }
}

fn disable_picker_buttons(mut buttons: Query<&mut Visibility, With<PickerButton>>) {
    for mut v in buttons.iter_mut() {
        *v = Visibility::Hidden;
    }
}
