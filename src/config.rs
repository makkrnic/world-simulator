use crate::world::WORLD_RESOLUTION;

const DEFAULT_CHUNK_RENDER_DISTANCE: i32 = 8;
const DEFAULT_MOVEMENT_SPEED: f32 = 50.0;

pub struct PlayerConfig {
  // radius of chunks around the player to render
  pub chunk_render_distance: i32,
  pub camera_distance: f32,
}

impl Default for PlayerConfig {
  fn default() -> Self {
    Self {
      chunk_render_distance: DEFAULT_CHUNK_RENDER_DISTANCE,
      camera_distance: 10.0,
    }
  }
}

pub struct MovementSettings {
  pub sensitivity: f32,
  pub speed: f32,
}

impl Default for MovementSettings {
  fn default() -> Self {
    Self {
      sensitivity: 0.00012,
      speed: DEFAULT_MOVEMENT_SPEED,
    }
  }
}
