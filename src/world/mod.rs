mod chunk_generator;
mod world;

pub use world::*;

/// WORLD_RESOLUTION defines ratio between coordinates and real in-game size
/// The vlaue 2 would specify that 1 "meter" spans over two full coordinate
/// system units (e.g. [0.0, 2.0) )
/// It was introduced to allow for more detailed terrain.
pub const WORLD_RESOLUTION: i32 = 2;

const BASE_CHUNK_SIZE_X: i32 = 16;
const BASE_CHUNK_SIZE_Z: i32 = 16;
const BASE_CHUNK_SIZE_Y: i32 = 256;

const CHUNK_SIZE_X: i32 = BASE_CHUNK_SIZE_X;
const CHUNK_SIZE_Z: i32 = BASE_CHUNK_SIZE_Z;
const CHUNK_SIZE_Y: i32 = BASE_CHUNK_SIZE_Y * WORLD_RESOLUTION;
