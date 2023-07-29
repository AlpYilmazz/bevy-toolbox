use std::time::Duration;

use animation::{
    curves, Animation, AnimationCurve, AnimationStep, Animator, Delay, Repeat, SequenceAnimator,
    TransformTranslationLens, TransformScaleLens,
};
use bevy::{prelude::*, window::PrimaryWindow};
use grid::{AsGridCoord, GridSettings};
use interpolation::EaseFunction;
use inventory::BaseInventory;
use items::{ItemCode, ItemPreview};
use utils::cursor_to_window_coord;

pub mod animation;
pub mod grid;
pub mod inventory;
pub mod items;
pub mod utils;

const DUMMY_IMAGE_PATH: &'static str = "happy-tree.png";
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

#[derive(Resource)]
pub struct DummyImage(pub Handle<Image>);

pub fn spawn_initial(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let dummy_image_handle = asset_server.load(DUMMY_IMAGE_PATH);
    commands.insert_resource(DummyImage(dummy_image_handle.clone()));

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

    // Spawn the dummy image for reference
    let window_padding = 40.0; // TODO: global?
    let pos = Vec3::new(
        -window_w / 2.0 + window_padding,
        -window_h / 2.0 + window_padding,
        0.5,
    );
    let scale = Vec3::new(1.0, 1.0, 1.0);
    let pos1 = pos;
    let pos2 = pos1 + Vec3::new(250.0, 250.0, 0.0);
    let pos3 = pos2 + Vec3::new(200.0, 0.0, 0.0);
    let scale1 = scale;
    let scale2 = Vec3::new(2.0, 2.0, 1.0);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            texture: dummy_image_handle,
            transform: Transform::from_translation(pos).with_scale(scale),
            ..Default::default()
        },
        // Animator::new(
        //     Animation {
        //         duration: Duration::from_secs(2),
        //         curve: AnimationCurve::new(curves::second_order),
        //     },
        //     Repeat::Always,
        //     TransformTranslationLens {
        //         start: pos1,
        //         end: pos2,
        //     },
        // ),
        SequenceAnimator::new(
            vec![
                AnimationStep::Animation(
                    Animation {
                        duration: Duration::from_secs(2),
                        curve: EaseFunction::QuadraticInOut.into(),
                    },
                    TransformTranslationLens {
                        start: pos1,
                        end: pos2,
                    },
                ),
                AnimationStep::Delay(Delay {
                    duration: Duration::from_secs(2),
                }),
                AnimationStep::Animation(
                    Animation {
                        duration: Duration::from_secs(2),
                        curve: AnimationCurve::Linear,
                    },
                    TransformTranslationLens {
                        start: pos2,
                        end: pos3,
                    },
                ),
            ],
            Repeat::Mirrored,
        ),
        Animator::new(
            Animation {
                duration: Duration::from_secs(3),
                curve: EaseFunction::BounceInOut.into(),
            },
            Repeat::Mirrored,
            TransformScaleLens {
                start: scale1,
                end: scale2,
            }
        )
    ));
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
    grid_settings: Res<GridSettings>,
    inventory: Res<BaseInventory>,
    mut preview_items: Query<(&ItemCode, &mut Transform, &mut Visibility), With<ItemPreview>>,
) {
    let grid_size = grid_settings.size;

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
                    let grid_translation = cursor_in_window
                        .as_grid_coord(grid_size)
                        .translation(grid_size);
                    transform.translation.x = grid_translation.x;
                    transform.translation.y = grid_translation.y;
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
