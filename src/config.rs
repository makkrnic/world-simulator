const DEFAULT_CHUNK_RENDER_DISTANCE: u8 = 2;

pub struct PlayerConfig {
    // radius of chunks around the player to render
    pub chunk_render_distance: u8,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            chunk_render_distance: DEFAULT_CHUNK_RENDER_DISTANCE,
        }
    }
}