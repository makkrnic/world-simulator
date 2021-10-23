const DEFAULT_CHUNK_RENDER_DISTANCE: i32 = 12;

pub struct PlayerConfig {
  // radius of chunks around the player to render
  pub chunk_render_distance: i32,
}

impl Default for PlayerConfig {
  fn default() -> Self {
    Self {
      chunk_render_distance: DEFAULT_CHUNK_RENDER_DISTANCE,
    }
  }
}
