use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::lens::TransformScaleLens;
use bevy_tweening::Animator;
use bevy_tweening::EaseFunction;
use bevy_tweening::Tracks;
use bevy_tweening::Tween;
use bevy_tweening::Tweenable;
use furuyoni_lib::rules::cards::Card;
use furuyoni_lib::rules::cards::CardSelector;
use furuyoni_lib::rules::events::GameEvent;
use furuyoni_lib::rules::states::StateView;
use super::spread_plugin;
use super::spread_plugin::Spread;
use super::BoardError;
use super::CardInspectPosition;
use super::CardObject;
use super::OpenCardObject;
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
                    let wait_animation = ctx.run_on_main_thread(move |ctx| {
                        let world = ctx.world;
                        let card_id = card_entity(from, world, me, card);
                        let slot_id = slot_entity(from, to, world, me);
                        animate_card(world, card_id, slot_id, me, from, to.position)
                    }).await;

                    tokio::time::sleep(wait_animation).await;
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

fn animate_card(
    world: &mut World,
    card_id: Entity,
    dest_slot_id: Entity,
    me: PlayerPos,
    from: CardsPosition,
    to: CardsPosition
) -> Duration {
    // calculate the global transforms.
    let (card_global, slot_global) = world.run_system_once(
        move |transform_helper: TransformHelper| {
            let card_global = transform_helper.compute_global_transform(card_id).unwrap();
            let slot_global = transform_helper.compute_global_transform(dest_slot_id).unwrap();

            (card_global, slot_global)
        }
    );

    // Put the card as a child of the slot while retaining the card's global position.
    // Note that set_parent_in_place won't work because the 'GlobalPosition's are not yet evaluated.
    let card_local = world.run_system_once(
        move |mut commands: Commands, mut transforms: Query<&mut Transform>| {
            let mut card_local = transforms.get_mut(card_id).unwrap();
            *card_local = card_global.reparented_to(&slot_global);
            commands.entity(card_id).set_parent(dest_slot_id);
            *card_local
        }
    );

    // Add appropriate tweens.
    world.run_system_once(
        move |
            mut commands: Commands,
            card_inspect: Query<(&CardInspectPosition, &GlobalTransform)>
        | {
            let (wait_duration, animator) = match (from, to) {
                (CardsPosition::Deck(p1), CardsPosition::Hand(p2)) if p1 == me && p2 == me => {
                    // draw.
                    let (_, card_inspect_global) = card_inspect.single();

                    let card_inspect_local = card_inspect_global.reparented_to(&slot_global);

                    (
                        Duration::from_millis(900),
                        Animator::new(
                            bevy_tweening::Sequence
                                ::with_capacity(2)
                                .then(
                                    transform_tween(
                                        &card_local,
                                        &card_inspect_local,
                                        Duration::from_millis(600)
                                    )
                                )
                                .then(
                                    transform_tween(
                                        &card_inspect_local,
                                        &Transform::IDENTITY,
                                        Duration::from_millis(600)
                                    )
                                )
                        ),
                    )
                }
                _ =>
                    (
                        Duration::from_millis(200),
                        Animator::new(
                            transform_tween(
                                &card_local,
                                &Transform::IDENTITY,
                                Duration::from_secs(1)
                            )
                        ),
                    ),
            };

            commands.entity(card_id).insert(animator);
            wait_duration
        }
    )
}

fn transform_tween(
    start: &Transform,
    end: &Transform,
    duration: Duration
) -> impl Tweenable<Transform> + 'static {
    let tween_translation = Tween::new(EaseFunction::QuadraticOut, duration, TransformPositionLens {
        start: start.translation,
        end: end.translation,
    });

    let tween_scale: Tween<Transform> = Tween::new(
        EaseFunction::QuarticOut,
        duration,
        TransformScaleLens {
            start: start.scale,
            end: end.scale,
        }
    );
    bevy_tweening::Tracks::new([tween_translation, tween_scale])
}

pub(crate) fn slot_entity(
    from: CardsPosition,
    to: CardSelector,
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
                        // FIXME: Refactor - group delay calculations together.
                        Duration::from_secs_f32(match from {
                            CardsPosition::Deck(p) if p == me => 0.8,
                            _ => 0.25,
                        })
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

pub(crate) fn card_entity(
    from: CardsPosition,
    world: &mut World,
    me: PlayerPos,
    card: Option<Card>
) -> Entity {
    let owner = from.player_pos();
    let spawn_from = match from {
        CardsPosition::Deck(p) =>
            world.run_system_once(
                move |mut commands: Commands, deck_objects: Query<(Entity, &DeckObject)>| {
                    let (deck_id, _) = deck_objects
                        .iter()
                        .find(|&(_, d)| { d.relative_pos.into_absolute(me) == p })
                        .unwrap();

                    deck_id
                }
            ),
        CardsPosition::Hand(_) => todo!(),
        CardsPosition::Discards(_) => todo!(),
        CardsPosition::Playing(_) | CardsPosition::Enhancements(_) | CardsPosition::Played(_) =>
            panic!("Impossible event."),
    };

    world.run_system_once(move |mut commands: Commands, asset_server: Res<AssetServer>| {
        let texture = match card {
            Some(_) => asset_server.load("sprites/cardfront_empty.png"),
            None => asset_server.load("sprites/cardback_normal.png"),
        };

        let mut new = commands.spawn((
            SpriteBundle {
                texture,
                ..default()
            },
            CardObject::new(owner),
        ));
        if let Some(card) = card {
            new.insert(OpenCardObject::new(card));
        }
        new.set_parent(spawn_from).id()
    })
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
