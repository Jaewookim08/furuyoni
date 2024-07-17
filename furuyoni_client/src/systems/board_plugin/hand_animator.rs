use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{ lens::TransformPositionLens, Animator, EaseFunction, Tween };

#[derive(Debug, Component, Clone)]
pub(crate) struct HandAnimation {
    max_breadth: f32,
    // Card counts for reaching the max breadth
    max_breadth_card_count: i32,
}

impl HandAnimation {
    pub fn new(max_breadth: f32, max_breadth_card_count: i32) -> Self {
        Self { max_breadth, max_breadth_card_count }
    }
}

pub(crate) fn add_hand_child(
    mut commands: Commands,
    entity: Entity,
    hand_animation: &HandAnimation,
    op_children: Option<&Children>
) -> Entity {
    let hand_size = match op_children {
        Some(children) => children.len(),
        None => 0,
    };

    let new = commands
        .spawn(
            SpatialBundle::from_transform(
                Transform::from_translation(
                    card_local_position(hand_animation, hand_size, hand_size + 1)
                )
            )
        )
        .id();
    commands.entity(entity).add_child(new);

    new
}

pub(crate) fn animate_hand_cards(
    mut commands: Commands,
    hand_query: Query<(&HandAnimation, &Children), Changed<Children>>,
    transforms: Query<&Transform>
) {
    // TODO: delay animations only when a card is added to show more natural movements.
    for (hand_animation, children) in hand_query.iter() {
        let hand_size = children.len();
        for (i, &child) in children.iter().enumerate() {
            let tween: Tween<Transform> = Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    // start from the child's initial position.
                    start: transforms.get(child).unwrap().translation,
                    end: card_local_position(hand_animation, i, hand_size),
                }
            );
            commands.entity(child).insert(Animator::new(tween));
        }
    }
}

fn card_local_position(hand_animation: &HandAnimation, i: usize, hand_size: usize) -> Vec3 {
    if hand_size == 0 {
        assert!(i == 0);
        return Vec3::new(0.0, 0.0, 0.0);
    }
    assert!(0 < hand_animation.max_breadth_card_count);
    // calculate hand breadth using the quadratic out function.
    let p = (((hand_size - 1) as f32) / ((hand_animation.max_breadth_card_count - 1) as f32)).clamp(
        0.0,
        1.0
    );
    let eased = -(p * (p - 2.0));
    let breadth = hand_animation.max_breadth * eased;

    let x =
        -breadth / 2.0 +
        breadth * (if hand_size <= 1 { 0.0 } else { (i as f32) / ((hand_size - 1) as f32) });

    Vec3::new(x, 0.0, 0.1 * (i as f32))
}
