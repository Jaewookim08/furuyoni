use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_tokio_tasks::TaskContext;
use furuyoni_lib::rules::player_actions::{BasicAction, MainPhaseAction};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct PickerPlugin;

pub async fn pick_basic_action(
    ctx: TaskContext,
    allowed_basic_actions: Arc<Vec<BasicAction>>,
    allow_skip: bool,
) -> Option<BasicAction> {
    let picked = pick_anything(ctx, allowed_basic_actions, allow_skip).await;

    match picked {
        Pickable::Skip => return None,
        Pickable::BasicActionButton(action) => return Some(action),
        _ => todo!("retry?"),
    }

    todo!()
}

#[derive(Resource)]
struct PickerCallBack {
    sender: Option<oneshot::Sender<Pickable>>,
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
            .add_systems(
                Update,
                (poll_pickers.run_if(resource_exists::<PickerCallBack>()),),
            );
    }
}

#[derive(Debug)]
enum Pickable {
    Skip,
    BasicActionButton(BasicAction),
}

async fn pick_anything(
    mut ctx: TaskContext,
    allowed_basic_actions: Arc<Vec<BasicAction>>,
    allow_skip: bool,
) -> Pickable {
    let (tx, rx) = oneshot::channel();

    ctx.run_on_main_thread(move |ctx| {
        ctx.world
            .insert_resource(PickerCallBack { sender: Some(tx) });
        ctx.world
            .run_system_once(enable_pickers_with(allowed_basic_actions, allow_skip));
    })
    .await;

    let ret = rx.await.expect(
        "todo: Picker system failed. Maybe the current request have been cancelled by another one.",
    );

    ctx.run_on_main_thread(move |ctx| {
        ctx.world.run_system_once(disable_picker_buttons);
    })
    .await;

    ret
}

fn enable_pickers_with(
    enabled_basic_actions: Arc<Vec<BasicAction>>,
    enable_skip: bool,
) -> impl Fn(
    ParamSet<(
        Query<&mut Visibility, With<SkipButton>>,
        Query<(&BasicActionButton, &mut Visibility)>,
    )>,
) {
    move |mut set: ParamSet<(
        Query<&mut Visibility, With<SkipButton>>,
        Query<(&BasicActionButton, &mut Visibility)>,
    )>| {
        for mut v in set.p0().iter_mut() {
            *v = if enable_skip {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }

        for (ba, mut v) in set.p1().iter_mut() {
            *v = if enabled_basic_actions.contains(&ba.action) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            }
        }
    }
}

fn poll_pickers(
    mut callback: ResMut<PickerCallBack>,
    basic_action_buttons: Query<(&Interaction, &BasicActionButton), Changed<Interaction>>,
    skip_buttons: Query<&Interaction, (Changed<Interaction>, With<SkipButton>)>,
) {
    if callback.sender.is_none() {
        return;
    }

    let picked = 'picked: {
        for (interaction, ba) in basic_action_buttons.iter() {
            match interaction {
                // Todo: bevy ui update되면 released 처리 방식 따라 수정.
                Interaction::Pressed => {
                    break 'picked Some(Pickable::BasicActionButton(ba.action));
                }
                _ => {}
            }
        }

        for interaction in skip_buttons.iter() {
            match interaction {
                Interaction::Pressed => break 'picked Some(Pickable::Skip),
                _ => (),
            }
        }

        None
    };

    if let Some(picked) = picked {
        let sender = callback.sender.take().unwrap();
        sender.send(picked).expect("todo");
    }
}

fn disable_picker_buttons(
    mut buttons: Query<&mut Visibility, Or<(With<SkipButton>, With<BasicActionButton>)>>,
) {
    for mut v in buttons.iter_mut() {
        *v = Visibility::Hidden;
    }
}
