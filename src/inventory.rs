use std::array;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::items::{Item, ItemCode, ItemImage, ItemPreview};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct BaseInventory(pub Inventory<9>);

#[derive(Resource)]
pub struct Inventory<const N: usize> {
    items: [Option<Item>; N], // use 1-indexed
    selected: usize,          // 0: no selection
}

impl<const N: usize> Default for Inventory<N> {
    fn default() -> Self {
        Self {
            items: array::from_fn(|_i| None),
            selected: 0,
        }
    }
}

// TODO: perform more bound checks (upper-bound)
impl<const N: usize> Inventory<N> {
    pub fn selected_slot(&self) -> Option<usize> {
        if self.selected == 0 {
            return None;
        }
        Some(self.selected)
    }

    pub fn clear_selection(&mut self) {
        self.selected = 0;
    }

    pub fn select_item(&mut self, selection: usize) {
        self.selected = selection;
    }

    pub fn selected_item(&self) -> Option<&Item> {
        if self.selected == 0 {
            return None;
        }
        self.items[self.selected - 1].as_ref()
    }

    /// slot: 1-indexed
    pub fn get_item(&self, slot: usize) -> Option<&Item> {
        if slot == 0 {
            return None;
        }
        self.items[slot - 1].as_ref()
    }

    /// slot: 1-indexed
    pub fn put_item(&mut self, slot: usize, item: Item) {
        self.items[slot - 1] = Some(item);
    }

    /// slot: 1-indexed
    pub fn remove_item(&mut self, slot: usize) -> Option<Item> {
        let item = self.items[slot - 1].clone();
        self.items[slot - 1] = None;
        item
    }
}

#[derive(Component)]
pub struct BaseInventoryBackground;

#[derive(Component)]
pub struct InventorySlotBackground {
    pub base: Entity,
    pub slot: usize,
}

#[derive(Component)]
pub struct InventorySlot {
    pub base: Entity,
    pub slot: usize,
}

#[derive(Resource, Deref, DerefMut)]
pub struct BaseInventorySettings(pub InventorySettings);

pub struct InventorySettings {
    pub w_padding: f32,
    pub w_mid_step: f32,
    pub h_padding: f32,
    // pub h_mid_step: f32,
    pub slot_margin: f32,
    pub slot_size: f32,
}

pub fn spawn_base_inventory(
    mut commands: Commands,
    settings: Res<BaseInventorySettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = primary_window.single();
    let window_h = primary_window.height();
    let _window_w = primary_window.width();
    let window_padding = 40.0;

    let n_slots = 9;
    let InventorySettings {
        w_padding,
        w_mid_step,
        h_padding,
        // h_mid_step,
        slot_margin,
        slot_size,
    } = settings.0;

    let pos = Vec2::new(0.0, -(window_h / 2.0) + window_padding);

    let w_total =
        (2.0 * w_padding) + (n_slots as f32 * slot_size) + ((n_slots - 1) as f32 * w_mid_step);

    let h_total = (2.0 * h_padding) + slot_size;

    let inventory_background = commands
        .spawn((
            BaseInventoryBackground,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GRAY,
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 42.0))
                    .with_scale(Vec3::new(w_total, h_total, 1.0)),
                visibility: Visibility::Visible,
                ..Default::default()
            },
        ))
        .id();

    trace!("{w_total}-{h_total}");
    trace!("---");
    let x_start = pos.x - (w_total / 2.0) + w_padding + (slot_size / 2.0);
    for i in 0..n_slots {
        let x = x_start + (i as f32 * (slot_size + w_mid_step));
        let y = pos.y;
        trace!("{x}-{y}");

        commands.spawn((
            InventorySlotBackground {
                base: inventory_background,
                slot: i + 1,
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.9, 0.9, 0.9, 1.0),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x, y, 43.0))
                    .with_scale(Vec3::new(slot_size, slot_size, 1.0)),
                visibility: Visibility::Visible,
                ..Default::default()
            },
        ));

        commands.spawn((
            InventorySlot {
                base: inventory_background,
                slot: i + 1,
            },
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(slot_size - slot_margin, slot_size - slot_margin)),
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x, y, 44.0)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
        ));
    }
    // inventory_background.with_children(|cb| {
    //     for i in 0..n_slots {
    //         let x = x_start + (i as f32 * (slot_size + w_mid_step));
    //         let y = pos.y;
    //         trace!("{x}-{y}");
    //         cb.spawn((
    //             InventorySlot,
    //             SpriteBundle {
    //                 sprite: Sprite {
    //                     color: Color::rgba(0.9, 0.9, 0.9, 1.0),
    //                     ..Default::default()
    //                 },
    //                 transform: Transform::from_translation(Vec3::new(x, y, 11.0))
    //                     * Transform::from_scale(Vec3::new(slot_size, slot_size, 1.0)),
    //                 visibility: Visibility::Visible,
    //                 ..Default::default()
    //             },
    //         ));
    //     }
    // });
}

pub fn render_items_in_base_inventory(
    inventory: Res<BaseInventory>,
    // images: Res<Assets<Image>>,
    preview_items: Query<(&ItemCode, &ItemImage), With<ItemPreview>>,
    mut slot_items: Query<(&InventorySlot, &mut Handle<Image>, &mut Visibility)>,
) {
    for (slot, mut slot_image, mut visibility) in slot_items.iter_mut() {
        if let Some(item) = &inventory.get_item(slot.slot) {
            let Some((_, item_image)) = preview_items
                .iter()
                .find(|(item_code, _)| **item_code == item.code)
            else {
                continue;
            };
            // trace!("Item in slot {}, code: {}", slot.slot, item.code.0);
            *slot_image = item_image.0.clone();
            *visibility = Visibility::Visible;
        }
    }
}
