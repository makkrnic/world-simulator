mod chunk_generator;
mod world;

pub use world::*;

const CHUNK_SIZE_X: i32 = 16;
const CHUNK_SIZE_Z: i32 = 16;
const CHUNK_SIZE_Y: i32 = 256;