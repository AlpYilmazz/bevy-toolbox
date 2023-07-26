use bevy::prelude::{IVec2, Resource, UVec2, Vec2, Vec3};

#[derive(Resource)]
pub struct GridSettings {
    pub size: u32,
}
// TODO: handle negative
// Grid index
#[derive(Debug)]
pub struct GridCoord {
    coord: UVec2,
    quad: IVec2, // (x: +-1, y: +-1)
}

impl GridCoord {
    /// Translation of the grid center in 2D space
    #[inline]
    pub fn translation(&self, grid_size: u32) -> Vec2 {
        Vec2 {
            x: (self.quad.x * grid_size as i32 * self.coord.x as i32) as f32
                + ((self.quad.x * grid_size as i32) as f32 / 2.0),
            y: (self.quad.y * grid_size as i32 * self.coord.y as i32) as f32
                + ((self.quad.y * grid_size as i32) as f32 / 2.0),
        }
    }

    /// Translation of the grid center in 3D space with z coordinate
    #[inline]
    pub fn translation_with_z(&self, grid_size: u32, z: f32) -> Vec3 {
        let translation_xy = self.translation(grid_size);
        Vec3 {
            x: translation_xy.x,
            y: translation_xy.y,
            z,
        }
    }
}

pub trait AsGridCoord {
    fn as_grid_coord(&self, grid_size: u32) -> GridCoord;
}
impl AsGridCoord for Vec2 {
    fn as_grid_coord(&self, grid_size: u32) -> GridCoord {
        GridCoord {
            coord: UVec2 {
                x: (self.x.abs() as u32) / grid_size as u32,
                y: (self.y.abs() as u32) / grid_size as u32,
            },
            quad: IVec2 {
                x: self.x.signum() as i32,
                y: self.y.signum() as i32,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        math::Vec3Swizzles,
        prelude::{Vec2, Vec3},
    };

    use super::AsGridCoord;

    struct TestPair {
        pub translation: Vec2,
        pub grid_translation: Vec2,
    }

    #[test]
    fn find_grid_coord() {
        let grid_size = 10;

        let tests = [
            TestPair {
                translation: Vec2::new(27.0, 41.4),
                grid_translation: Vec2::new(25.0, 45.0),
            },
            TestPair {
                translation: Vec2::new(-27.0, 41.4),
                grid_translation: Vec2::new(-25.0, 45.0),
            },
            TestPair {
                translation: Vec2::new(-27.0, -41.4),
                grid_translation: Vec2::new(-25.0, -45.0),
            },
            TestPair {
                translation: Vec2::new(27.0, -41.4),
                grid_translation: Vec2::new(25.0, -45.0),
            },
            TestPair {
                translation: Vec2::new(0.0, 0.0),
                grid_translation: Vec2::new(5.0, 5.0),
            },
            TestPair {
                translation: Vec2::new(0.001, 0.0),
                grid_translation: Vec2::new(5.0, 5.0),
            },
            TestPair {
                translation: Vec2::new(-0.001, 0.0),
                grid_translation: Vec2::new(-5.0, 5.0),
            },
            TestPair {
                translation: Vec2::new(-0.001, -0.001),
                grid_translation: Vec2::new(-5.0, -5.0),
            },
            TestPair {
                translation: Vec2::new(0.001, -0.001),
                grid_translation: Vec2::new(5.0, -5.0),
            },
        ];

        for TestPair {
            translation,
            grid_translation,
        } in tests
        {
            let grid_coord = translation.as_grid_coord(grid_size);
            let grid_translation_found = grid_coord.translation(grid_size);
            
            println!("{:?}", grid_coord);
            assert_eq!(grid_translation_found, grid_translation);
        }
    }
}
