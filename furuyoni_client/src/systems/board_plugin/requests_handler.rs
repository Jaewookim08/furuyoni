use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::Animator;
use bevy_tweening::EaseFunction;
use bevy_tweening::Tween;
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::states::StateView;
use super::spread_plugin;
use super::spread_plugin::Spread;
use super::BoardError;
use super::DeckObject;
use super::HandObject;
use furuyoni_lib::rules::cards::CardsPosition;
use furuyoni_lib::rules::events::UpdateGameState;
use bevy_tokio_tasks::TaskContext;
use super::SelfPlayerPos;
use super::BoardState;
use furuyoni_lib::rules::PlayerPos;

pub(crate) fn initialize_board(world: &mut World, state: StateView, me: PlayerPos) {
    world.insert_resource(BoardState { 0: state });
    world.insert_resource(SelfPlayerPos { 0: me });
}

/// display the event in the board and return if the game has ended.
pub(crate) async fn apply_event(
    ctx: &TaskContext,
    event: GameEvent,
    me: PlayerPos
) -> Result<(), BoardError> {
    match event {
        GameEvent::StateUpdated(update) => {
            ctx.run_on_main_thread(
                move |ctx| -> Result<(), BoardError> {
                    let mut state = ctx.world.get_resource_mut::<BoardState>().unwrap();
                    state.0.apply_update(update)?;
                    Ok(())
                }
            ).await?;

            match update {
                UpdateGameState::TransferCardFromHidden { from, to, card } => {
                    ctx.run_on_main_thread(move |ctx| {
                        let world = ctx.world;
                        let card_id = get_card_entity(from, world, me);
                        let slot_id = get_slot_entity(to, world, me);
                        animate_card(world, card_id, slot_id);
                    }).await;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                _ => /* TODO */ (),
            }
        }
        GameEvent::PerformBasicAction { .. } => {/* Todo */}
        GameEvent::GameEnd { result: _ } => {
            // TODO:
        }
    }
    Ok(())
}

fn animate_card(world: &mut World, card_id: Entity, slot_id: Entity) {
    world.run_system_once(
        move |
            mut commands: Commands,
            mut transform_params: ParamSet<(TransformHelper, Query<&mut Transform>)>
        | {
            // Put the card as a child of the slot while retaining the card's global position.
            // Note that set_parent_in_place doesn't work because the 'GlobalPosition's are not yet evaluated.
            let card_global = transform_params.p0().compute_global_transform(card_id).unwrap();
            let slot_global = transform_params.p0().compute_global_transform(slot_id).unwrap();
            let mut transforms = transform_params.p1();
            let mut card_local = transforms.get_mut(card_id).unwrap();
            *card_local = card_global.reparented_to(&slot_global);

            commands.entity(card_id).set_parent(slot_id);

            let tween_translation = Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    start: card_local.translation,
                    end: Vec3::ZERO,
                }
            );

            let tween_scale = Tween::new(
                EaseFunction::QuarticOut,
                Duration::from_secs(1),
                TransformScaleLens {
                    start: card_local.scale,
                    end: Vec3::ONE,
                }
            );

            let tween = bevy_tweening::Tracks::new([tween_translation, tween_scale]);

            commands.entity(card_id).insert(Animator::new(tween));
        }
    );
}

pub(crate) fn get_slot_entity(
    to: furuyoni_lib::rules::cards::CardSelector,
    world: &mut World,
    me: PlayerPos
) -> Entity {
    match to.position {
        CardsPosition::Hand(p) => {
            world.run_system_once(
                move |
                    commands: Commands,
                    hand_objects: Query<(Entity, &HandObject, &Spread, Option<&Children>)>
                | {
                    let (hand_id, _, hand_animation, children) = hand_objects
                        .iter()
                        .find(|&(_, h, _, _)| { h.relative_pos.into_absolute(me) == p })
                        .unwrap();
                    let new = spread_plugin::add_spread_child(
                        commands,
                        hand_id,
                        hand_animation,
                        children,
                        0.35,
                    );
                    new
                }
            )
        }
        CardsPosition::Deck(_) => todo!(),
        CardsPosition::Playing(_) => todo!(),
        CardsPosition::Enhancements(_) => todo!(),
        CardsPosition::Played(_) => todo!(),
        CardsPosition::Discards(_) => todo!(),
    }
}

pub(crate) fn get_card_entity(
    from: CardsPosition,
    world: &mut World,
    me: PlayerPos,
) -> Entity {
    match from {
        CardsPosition::Deck(p) =>
            world.run_system_once(
                move |
                    mut commands: Commands,
                    asset_server: Res<AssetServer>,
                    deck_objects: Query<(Entity, &DeckObject)>
                | {
                    let (deck_id, _) = deck_objects
                        .iter()
                        .find(|&(_, d)| { d.relative_pos.into_absolute(me) == p })
                        .unwrap();

                    let card_id = commands
                        .spawn((
                            SpriteBundle {
                                texture: asset_server.load("sprites/cardback_normal.png"),
                                ..default()
                            },
                        ))
                        .set_parent(deck_id)
                        .id();
                    card_id
                }
            ),
        CardsPosition::Hand(_) => todo!(),
        CardsPosition::Discards(_) => todo!(),
        CardsPosition::Playing(_) | CardsPosition::Enhancements(_) | CardsPosition::Played(_) =>
            panic!("Impossible event."),
    }
}

pub(crate) async fn check_game_state(ctx: &TaskContext, state: StateView) {
    ctx.run_on_main_thread(move |ctx| {
        let resource = ctx.world
            .get_resource::<BoardState>()
            .expect("Resource BoardState is missing.");
        if resource.0 != state {
            eprintln!("Error: state mismatch.");
            eprintln!("server state: {:?}", state);
            eprintln!("client state: {:?}", resource.0);
            todo!("handle state mismatch: resynchronize...")
        }
    }).await;
}
