use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{ lens::TransformPositionLens, Animator, EaseFunction, Tween };

#[derive(Debug, Component, Clone, Reflect)]
pub(crate) struct Spread {
    max_breadth: f32,
    /// Elements counts for reaching the max breadth
    max_breadth_elements: i32,
}

impl Spread {
    pub fn new(max_breadth: f32, max_breadth_elements: i32) -> Self {
        Self { max_breadth, max_breadth_elements }
    }
}

pub(crate) struct SpreadPlugin;

impl Plugin for SpreadPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Spread>()
        .add_event::<SpreadElementActivatedEvent>()
        .add_systems(PostUpdate, (tick_spread, animate_spread).chain());
    }
}

#[derive(Debug, Component, Clone)]
struct SpreadElement {
    /// Timer to count until the SpreadElement is active. This allows to add delay for spread animations when a new element is added.
    activate_timer: Timer,
}

#[derive(Event)]
struct SpreadElementActivatedEvent(Entity /* The parent Spread. */);

fn tick_spread(
    time: Res<Time>,
    mut spread_elements: Query<(&mut SpreadElement, &Parent)>,
    mut ev_activated: EventWriter<SpreadElementActivatedEvent>
) {
    for (mut se, parent) in spread_elements.iter_mut() {
        se.activate_timer.tick(time.delta());

        if se.activate_timer.just_finished() {
            ev_activated.send(SpreadElementActivatedEvent(parent.get()));
        }
    }
}

pub(crate) fn add_spread_child(
    mut commands: Commands,
    entity: Entity,
    spread: &Spread,
    op_children: Option<&Children>,
    delay: f32
) -> Entity {
    let children_count = match op_children {
        Some(children) => children.len(),
        None => 0,
    };

    let new = commands
        .spawn((
            SpatialBundle::from_transform(
                Transform::from_translation(
                    element_position(spread, children_count, children_count + 1)
                )
            ),
            SpreadElement { activate_timer: Timer::from_seconds(delay, TimerMode::Once) },
        ))
        .id();
    commands.entity(entity).add_child(new);

    new
}

fn animate_spread(
    mut commands: Commands,
    spread_query: Query<(&Spread, &Children)>,
    element_query: Query<(Entity, &SpreadElement, &Transform)>,
    mut ev_changed: EventReader<SpreadElementActivatedEvent>
) {
    for ev in ev_changed.read() {
        let (spread, children) = spread_query
            .get(ev.0)
            .expect("A parent of a SpreadElement should have Spread as its component.");
        let elements: Vec<_> = children
            .iter()
            .map(|&child|
                element_query
                    .get(child)
                    .expect("All childs of a Spread should have SpreadElement as its component.")
            )
            .filter(|(_, se, _)| { se.activate_timer.finished() })
            .collect();

        let elements_count = elements.len();
        for (i, &(child, _, child_transform)) in elements.iter().enumerate() {
            let tween: Tween<Transform> = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(650),
                TransformPositionLens {
                    // start from the child's initial position.
                    start: child_transform.translation,
                    end: element_position(spread, i, elements_count),
                }
            );
            commands.entity(child).insert(Animator::new(tween));
        }
    }
}

fn element_position(spread: &Spread, i: usize, elements_count: usize) -> Vec3 {
    if elements_count == 0 {
        assert!(i == 0);
        return Vec3::new(0.0, 0.0, 0.0);
    }
    assert!(0 < spread.max_breadth_elements);
    // calculate hand breadth using the quadratic out function.
    let p = (((elements_count - 1) as f32) / ((spread.max_breadth_elements - 1) as f32)).clamp(
        0.0,
        1.0
    );
    let eased = -(p * (p - 2.0));
    let breadth = spread.max_breadth * eased;

    let x =
        -breadth / 2.0 +
        breadth *
            (if elements_count <= 1 { 0.0 } else { (i as f32) / ((elements_count - 1) as f32) });

    Vec3::new(x, 0.0, 0.1 * (i as f32))
}
