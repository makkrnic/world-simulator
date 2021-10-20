use bevy::prelude::Mut;
use building_blocks::core::{ExtentN, PointN};
use building_blocks::prelude::FillExtent;
use crate::world::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, Voxel};
use simdnoise::NoiseBuilder;

const LEVEL_SEED: i32 = 0;
const GROUND_LEVEL: u32 = 100;
const NOISE_GROUND_MAX_OFFSET: u32 = 50;

const NOISE_MIN: f32 = (GROUND_LEVEL - NOISE_GROUND_MAX_OFFSET) as f32;
const NOISE_MAX: f32 = (GROUND_LEVEL + NOISE_GROUND_MAX_OFFSET) as f32;

pub(crate) fn generate_chunk(mut chunk: Mut<Chunk>) {
    let (noise, _, _) = NoiseBuilder::fbm_2d_offset(
        (chunk.pos.x * CHUNK_SIZE_X) as f32,
        CHUNK_SIZE_X as usize,
        (chunk.pos.y * CHUNK_SIZE_Z) as f32,
        CHUNK_SIZE_Z as usize)
        .with_seed(LEVEL_SEED)
        .with_octaves(5)
        .with_freq(0.02)
        .generate();

    for z in 0..CHUNK_SIZE_Z {
        for x in 0..CHUNK_SIZE_X {
            let height = (noise.get((z * CHUNK_SIZE_X + x) as usize).unwrap() + 1.0) * (NOISE_MAX - NOISE_MIN) + NOISE_MIN;

            let block_height = (height.round() as i32).max(0).min(CHUNK_SIZE_Y -1);
            // println!("pos: ({}, {}) -> {}", x, z, block_height);

            // Put zeroth level
            chunk.block_data.fill_extent(
                &ExtentN::from_min_and_max(PointN([0; 3]), PointN([CHUNK_SIZE_X, 0, CHUNK_SIZE_Z])),
                Voxel {
                    attributes: [194, 178, 128, 255],
                },
            );

            chunk.block_data.fill_extent(
                &ExtentN::from_min_and_max(
                    PointN([x, 1, z]),
                    PointN([x, block_height, z]),
                ),
                Voxel {
                    attributes: [99, 146, 103, 255],
                },
            )
        }
    }
}