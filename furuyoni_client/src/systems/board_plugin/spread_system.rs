use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{ lens::TransformPositionLens, Animator, EaseFunction, Tween };

#[derive(Debug, Component, Clone)]
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

pub(crate) fn add_spread_child(
    mut commands: Commands,
    entity: Entity,
    spread: &Spread,
    op_children: Option<&Children>
) -> Entity {
    let elements_count = match op_children {
        Some(children) => children.len(),
        None => 0,
    };

    let new = commands
        .spawn(
            SpatialBundle::from_transform(
                Transform::from_translation(element_position(spread, elements_count, elements_count + 1))
            )
        )
        .id();
    commands.entity(entity).add_child(new);

    new
}

pub(crate) fn animate_spread(
    mut commands: Commands,
    hand_query: Query<(&Spread, &Children), Changed<Children>>,
    transforms: Query<&Transform>
) {
    // TODO: delay animations only when a card is added to show more natural movements.
    for (spread, children) in hand_query.iter() {
        let elements_count = children.len();
        for (i, &child) in children.iter().enumerate() {
            let tween: Tween<Transform> = Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    // start from the child's initial position.
                    start: transforms.get(child).unwrap().translation,
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
        breadth * (if elements_count <= 1 { 0.0 } else { (i as f32) / ((elements_count - 1) as f32) });

    Vec3::new(x, 0.0, 0.1 * (i as f32))
}
