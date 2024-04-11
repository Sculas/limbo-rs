use azalea_core::position::{BlockPos, Vec3};

/// A location in the world.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Location {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
}

impl Location {
    /// Creates a new location.
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Self {
        Self {
            x,
            y,
            z,
            yaw,
            pitch,
        }
    }

    /// Returns the X coordinate.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the Y coordinate.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the Z coordinate.
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Returns the yaw.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Returns the pitch.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Converts this location to a Vec3.
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Converts this location to a BlockPos.
    pub fn to_block_pos(&self) -> BlockPos {
        BlockPos {
            x: self.x.floor() as i32,
            y: self.y.floor() as i32,
            z: self.z.floor() as i32,
        }
    }
}
