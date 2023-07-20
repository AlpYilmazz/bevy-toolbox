use bevy::{prelude::*, window::PrimaryWindow};
use inventory::BaseInventory;
use items::{ItemCode, ItemPreview};
use utils::cursor_to_window_coord;

pub mod inventory;
pub mod items;
pub mod utils;

const BACKGROUND_COLOR: Color = Color::rgba(0.0, 180.0 / 255.0, 1.0, 1.0);

const NUMERIC_KEY_CODES: &'static [(KeyCode, usize)] = &[
    (KeyCode::Key0, 0),
    (KeyCode::Key1, 1),
    (KeyCode::Key2, 2),
    (KeyCode::Key3, 3),
    (KeyCode::Key4, 4),
    (KeyCode::Key5, 5),
    (KeyCode::Key6, 6),
    (KeyCode::Key7, 7),
    (KeyCode::Key8, 8),
    (KeyCode::Key9, 9),
];

pub fn spawn_initial(mut commands: Commands, primary_window: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());

    let primary_window = primary_window.single();
    let window_h = primary_window.height();
    let window_w = primary_window.width();

    // Spawn background
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: BACKGROUND_COLOR,
            anchor: bevy::sprite::Anchor::Center,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::new(window_w, window_h, 1.0)),
        ..Default::default()
    });
}

pub fn select_item(key: Res<Input<KeyCode>>, mut inventory: ResMut<BaseInventory>) {
    for (keycode, num) in NUMERIC_KEY_CODES.iter() {
        if key.pressed(*keycode) {
            inventory.select_item(*num);
        }
    }
}

pub fn show_selected_item(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    inventory: Res<BaseInventory>,
    mut preview_items: Query<(&ItemCode, &mut Transform, &mut Visibility), With<ItemPreview>>,
) {
    let primary_window = primary_window.single();
    let window_h = primary_window.height();
    let window_w = primary_window.width();
    let cursor = &primary_window.cursor_position();

    let selected_item = inventory.selected_item();
    for (item_code, mut transform, mut visibility) in preview_items.iter_mut() {
        *visibility = Visibility::Hidden;
        if let Some(selected_item) = selected_item {
            if item_code.eq(&selected_item.code) {
                *visibility = Visibility::Visible;
                if let Some(cursor) = cursor {
                    // debug!("{:?}", cursor);
                    let cursor_in_window =
                        cursor_to_window_coord(cursor.clone(), window_h, window_w);
                    transform.translation.x = cursor_in_window.x;
                    transform.translation.y = cursor_in_window.y;
                }
            }
        }
    }
}

pub fn place_selected_item(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    inventory: Res<BaseInventory>,
    preview_items: Query<(&ItemCode, &Sprite, &Transform), With<ItemPreview>>,
) {
    if !(mouse.just_pressed(MouseButton::Left)) {
        return;
    }
    let Some(selected_item) = inventory.selected_item() else {
        return;
    };
    let Some((_, sprite, transform)) = preview_items
        .iter()
        .find(|(item_code, _, _)| **item_code == selected_item.code) else {
            return;
        };
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: sprite.color.with_a(1.0),
            ..Default::default()
        },
        transform: transform.clone(),
        visibility: Visibility::Visible,
        ..Default::default()
    });
}

pub fn log_selected_item(
    inventory: Res<BaseInventory>,
    preview_items: Query<(&ItemCode, &Visibility), With<ItemPreview>>,
) {
    if let Some(item) = inventory.selected_item() {
        let visible = preview_items
            .iter()
            .find(|(item_code, _)| **item_code == item.code)
            .map(|(_, v)| v);
        info!("Selected: {} - {:?}", item.code.0, visible);
    }
}
