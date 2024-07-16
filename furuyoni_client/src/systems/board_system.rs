use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy_tokio_tasks::TaskContext;
use furuyoni_lib::rules::cards::{ Card, CardsPosition };
use furuyoni_lib::rules::events::{ GameEvent, UpdateGameState };
use furuyoni_lib::rules::states::{ InvalidGameViewUpdateError, PetalsPosition, StateView };
use furuyoni_lib::rules::PlayerPos;
use thiserror::Error;

pub(crate) struct BoardPlugin;

pub(crate) fn initialize_board(world: &mut World, state: StateView, me: PlayerPos) {
    world.insert_resource(BoardState { 0: state });
    world.insert_resource(SelfPlayerPos { 0: me });
}

#[derive(Debug, Error)]
pub(crate) enum BoardError {
    #[error("Tried to do an invalid update to the game state: {0}")] InvalidUpdate(
        #[from] InvalidGameViewUpdateError,
    ),
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
                        let card_id = get_card_entity(from, world, me, card);
                        let slot_id = get_slot_entity(to, world, me);
                        world.run_system_once(
                            move |
                                mut commands: Commands,
                                mut transform_params: ParamSet<
                                    (TransformHelper, Query<&mut Transform>)
                                >
                            | {
                                // Put the card as a child of the slot while retaining the card's global position.
                                // Note that set_parent_in_place doesn't work because the 'GlobalPosition's are not yet evaluated.
                                let card_global = transform_params.p0().compute_global_transform(card_id).unwrap();
                                let slot_global = transform_params.p0().compute_global_transform(slot_id).unwrap();
                                let mut transforms = transform_params.p1();
                                let mut card_local = transforms.get_mut(card_id).unwrap();
                                *card_local = card_global.reparented_to(&slot_global);
                                
                                commands.entity(card_id).set_parent(slot_id);
                            }
                        );
                    }).await;
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

fn get_slot_entity(
    to: furuyoni_lib::rules::cards::CardSelector,
    world: &mut World,
    me: PlayerPos
) -> Entity {
    match to.position {
        CardsPosition::Hand(p) => {
            world.run_system_once(
                move |mut commands: Commands, hand_objects: Query<(Entity, &HandObject)>| {
                    let (hand_id, _) = hand_objects
                        .iter()
                        .find(|&(_, h)| { h.relative_pos.into_absolute(me) == p })
                        .unwrap();
                    commands.spawn(TransformBundle::default()).set_parent(hand_id).id()
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

fn get_card_entity(from: CardsPosition, world: &mut World, me: PlayerPos, card: Card) -> Entity {
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
                            CardObject { card },
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

#[derive(Debug, Component)]
pub(crate) struct CardObject {
    card: Card,
}

impl CardObject {
    pub(crate) fn new(card: Card) -> Self {
        Self { card }
    }
}

#[derive(Debug, Component)]
pub(crate) struct HandObject {
    relative_pos: PlayerRelativePos,
}

impl HandObject {
    pub(crate) fn new(relative_pos: PlayerRelativePos) -> Self {
        Self { relative_pos }
    }
}

#[derive(Debug, Component)]
pub(crate) struct DeckObject {
    relative_pos: PlayerRelativePos,
}

impl DeckObject {
    pub(crate) fn new(relative_pos: PlayerRelativePos) -> Self {
        Self { relative_pos }
    }
}

#[derive(Resource)]
struct BoardState(pub StateView);

#[derive(Resource)]
struct SelfPlayerPos(pub PlayerPos);

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StateLabel>()
            .register_type::<StateStringPicker>()
            .register_type::<PlayerRelativePos>()
            .add_systems(
                Update,
                display_board
                    .run_if(resource_exists::<BoardState>)
                    .run_if(resource_exists::<SelfPlayerPos>)
            );
    }
}

fn display_board(
    state: Res<BoardState>,
    self_pos: Res<SelfPlayerPos>,
    mut query: Query<(&mut Text, &StateLabel)>
) {
    if state.is_changed() {
        for (mut text, state_label) in &mut query {
            text.sections[state_label.text_section_index].value = get_string(
                self_pos.0,
                &state.0,
                &state_label.picker
            );
        }
    }
}

#[derive(Debug, Reflect, Default, Copy, Clone)]
pub enum PlayerRelativePos {
    #[default]
    Me,
    Opponent,
}

impl PlayerRelativePos {
    pub fn into_absolute(self, me: PlayerPos) -> PlayerPos {
        match self {
            PlayerRelativePos::Me => me,
            PlayerRelativePos::Opponent => me.other(),
        }
    }
}

// Todo: refactor using PetalsPos.
#[derive(Debug, Copy, Clone, Reflect, Default)]
pub enum StateStringPicker {
    #[default]
    Turn,
    Vigor(PlayerRelativePos),
    PetalsCount(PetalsRelativePosition),
    CardsCount(CardsRelativePosition),
}

#[derive(Debug, Copy, Clone, Reflect)]
pub enum PetalsRelativePosition {
    Distance,
    Dust,
    Aura(PlayerRelativePos),
    Flare(PlayerRelativePos),
    Life(PlayerRelativePos),
}

impl PetalsRelativePosition {
    pub fn into_absolute(self, me: PlayerPos) -> PetalsPosition {
        match self {
            PetalsRelativePosition::Distance => PetalsPosition::Distance,
            PetalsRelativePosition::Dust => PetalsPosition::Dust,
            PetalsRelativePosition::Aura(p) => PetalsPosition::Aura(p.into_absolute(me)),
            PetalsRelativePosition::Flare(p) => PetalsPosition::Flare(p.into_absolute(me)),
            PetalsRelativePosition::Life(p) => PetalsPosition::Life(p.into_absolute(me)),
        }
    }
}

#[derive(Debug, Copy, Clone, Reflect)]
pub enum CardsRelativePosition {
    Hand(PlayerRelativePos),
    Playing(PlayerRelativePos),
    Deck(PlayerRelativePos),
    Enhancements(PlayerRelativePos),
    Played(PlayerRelativePos),
    Discards(PlayerRelativePos),
}

impl CardsRelativePosition {
    pub fn into_absolute(self, me: PlayerPos) -> CardsPosition {
        match self {
            CardsRelativePosition::Hand(p) => CardsPosition::Hand(p.into_absolute(me)),
            CardsRelativePosition::Playing(p) => CardsPosition::Playing(p.into_absolute(me)),
            CardsRelativePosition::Deck(p) => CardsPosition::Deck(p.into_absolute(me)),
            CardsRelativePosition::Enhancements(p) => {
                CardsPosition::Enhancements(p.into_absolute(me))
            }
            CardsRelativePosition::Played(p) => CardsPosition::Played(p.into_absolute(me)),
            CardsRelativePosition::Discards(p) => CardsPosition::Discards(p.into_absolute(me)),
        }
    }
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

fn get_string(me: PlayerPos, state: &StateView, picker: &StateStringPicker) -> String {
    let get_player = |rel_pos: &PlayerRelativePos| &state.player_states[rel_pos.into_absolute(me)];

    match picker {
        StateStringPicker::Turn => state.turn.to_string(),
        StateStringPicker::Vigor(rp) => get_player(rp).vigor.to_string(),
        StateStringPicker::PetalsCount(pos) => {
            state.petals(pos.into_absolute(me)).count.to_string()
        }
        StateStringPicker::CardsCount(pos) => {
            state.cards_view(pos.into_absolute(me)).len().to_string()
        }
    }
}
