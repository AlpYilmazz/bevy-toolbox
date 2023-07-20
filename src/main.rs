#![allow(unused)]

use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use bevy_toolbox::{
    inventory::{spawn_base_inventory, BaseInventory, BaseInventorySettings, InventorySettings},
    items::spawn_item_prototypes,
    log_selected_item, place_selected_item, select_item, show_selected_item, spawn_initial,
};

#[derive(Resource)]
struct Resolution {
    pub current: Vec2,
}

impl Default for Resolution {
    fn default() -> Self {
        Self {
            current: Self::SMALL,
        }
    }
}

impl Resolution {
    const FULLSCREEN: Vec2 = Vec2::new(1920.0, 1200.0);
    const SMALL: Vec2 = Vec2::new(1000.0, 500.0);

    pub fn toggle(&mut self) {
        if self.current == Self::FULLSCREEN {
            self.current = Self::SMALL;
        } else if self.current == Self::SMALL {
            self.current = Self::FULLSCREEN;
        }
    }
}

fn toggle_fullscreen(
    key: Res<Input<KeyCode>>,
    mut resolution: ResMut<Resolution>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = primary_window.single_mut();

    if key.just_pressed(KeyCode::Space) {
        resolution.toggle();
        let Vec2 { x, y } = resolution.current;
        let scale = primary_window.scale_factor();
        primary_window
            .resolution
            .set(x / scale as f32, y / scale as f32);
        primary_window.position = WindowPosition::Centered(MonitorSelection::Current);
        debug!(
            "Resolution: {:?}, Scale: {}",
            primary_window.resolution,
            primary_window.scale_factor()
        );
    }
}

fn init_window(
    resolution: Res<Resolution>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = primary_window.single_mut();
    let Vec2 { x, y } = resolution.current;
    let scale = primary_window.scale_factor();
    primary_window
        .resolution
        .set(x / scale as f32, y / scale as f32);
    primary_window.position = WindowPosition::Centered(MonitorSelection::Current);
    debug!(
        "Resolution: {:?}, Scale: {}",
        primary_window.resolution,
        primary_window.scale_factor()
    );
}

fn exit_on_close(key: Res<Input<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if key.pressed(KeyCode::Escape) {
        app_exit.send_default();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // mode: bevy::window::WindowMode::Windowed,
                // position: WindowPosition::Centered(MonitorSelection::Primary),
                // resolution: WindowResolution::new(Resolution::SMALL.x, Resolution::SMALL.y),
                ..Default::default()
            }),
            ..Default::default()
        }))
        // -- General --
        .init_resource::<Resolution>()
        .add_systems(PreStartup, init_window)
        .add_systems(PreUpdate, toggle_fullscreen)
        .add_systems(Update, exit_on_close)
        // -- Library Base --
        .add_systems(Startup, spawn_initial)
        // -- Inventory System --
        .init_resource::<BaseInventory>()
        .insert_resource(BaseInventorySettings(InventorySettings {
            w_padding: 5.0,
            w_mid_step: 4.0,
            h_padding: 3.0,
            slot_size: 50.0,
        }))
        .add_systems(Startup, spawn_item_prototypes)
        .add_systems(Startup, spawn_base_inventory)
        .add_systems(Update, select_item)
        .add_systems(Update, show_selected_item)
        .add_systems(Update, place_selected_item)
        // .add_systems(Update, log_selected_item)
        // ----- END -----
        .run();
}
