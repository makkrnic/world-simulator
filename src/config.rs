use crate::world::WORLD_RESOLUTION;

const DEFAULT_CHUNK_RENDER_DISTANCE: i32 = 4;
const DEFAULT_MOVEMENT_SPEED: f32 = 1.0;

pub struct PlayerConfig {
  // radius of chunks around the player to render
  pub chunk_render_distance: i32,
  pub camera_distance: f32,
}

impl Default for PlayerConfig {
  fn default() -> Self {
    Self {
      chunk_render_distance: DEFAULT_CHUNK_RENDER_DISTANCE,
      camera_distance: 10.0 * WORLD_RESOLUTION as f32,
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
