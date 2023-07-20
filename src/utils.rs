use bevy::prelude::Vec2;


pub fn cursor_to_window_coord(cursor: Vec2, window_h: f32, window_w: f32) -> Vec2 {
    Vec2 {
        x: cursor.x - (window_w / 2.0),
        y: -cursor.y + (window_h / 2.0),
    }
}