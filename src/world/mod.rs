mod chunk_generator;
mod world;

use bevy::diagnostic::DiagnosticId;
pub use world::*;

/// WORLD_RESOLUTION defines ratio between coordinates and real in-game size
/// The vlaue 2 would specify that 1 "meter" spans over two full coordinate
/// system units (e.g. [0.0, 2.0) )
/// It was introduced to allow for more detailed terrain.
pub const WORLD_RESOLUTION: i32 = 128;

pub const MAX_CHUNKS_GEN_PER_FRAME: i32 = 16;

const BASE_CHUNK_SIZE_X: i32 = 1;
const BASE_CHUNK_SIZE_Z: i32 = 1;
const BASE_CHUNK_SIZE_Y: i32 = 1;

const CHUNK_SIZE_X: i32 = BASE_CHUNK_SIZE_X * WORLD_RESOLUTION;
const CHUNK_SIZE_Z: i32 = BASE_CHUNK_SIZE_Z * WORLD_RESOLUTION;
const CHUNK_SIZE_Y: i32 = BASE_CHUNK_SIZE_Y * WORLD_RESOLUTION;

pub const CHUNK_GEN_TIME: DiagnosticId = DiagnosticId::from_u128(2048406892113231623654965);
pub const CHUNK_MESH_TIME: DiagnosticId = DiagnosticId::from_u128( 330101286740681123473332);
pub const VISIBLE_CHUNK_UPDATE_TIME: DiagnosticId = DiagnosticId::from_u128(3790910818159473405356513);
pub const CHUNK_LOAD_DATA_TIME: DiagnosticId = DiagnosticId::from_u128(759050099195211273636076);
