use bevy::prelude::*;

use crate::inventory::BaseInventory;

#[derive(Component, Clone, Copy)]
pub struct ItemPreview;

#[derive(Component, Clone, Copy, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemCode(pub usize);

#[derive(Clone)]
pub struct Item {
    pub code: ItemCode,
}

pub fn spawn_item_prototypes(mut commands: Commands, mut inventory: ResMut<BaseInventory>) {
    // 0: Rectangle item
    commands.spawn((
        ItemPreview,
        ItemCode(1),
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED.with_a(0.5),
                anchor: bevy::sprite::Anchor::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                .with_scale(Vec3::new(100.0, 20.0, 1.0)),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
    ));
    inventory.put_item(1, Item { code: ItemCode(1) });

    // 1: Square object
    commands.spawn((
        ItemPreview,
        ItemCode(2),
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN.with_a(0.5),
                anchor: bevy::sprite::Anchor::Center,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
                .with_scale(Vec3::new(20.0, 20.0, 1.0)),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
    ));
    inventory.put_item(2, Item { code: ItemCode(2) });
}
