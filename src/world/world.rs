use crate::config::PlayerConfig;
use crate::player::{Player, PlayerCamera};
use crate::world::chunk_generator::generate_chunk;
use crate::world::{CHUNK_GEN_TIME, CHUNK_LOAD_DATA_TIME, CHUNK_MESH_TIME, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, MAX_CHUNKS_GEN_PER_FRAME, VISIBLE_CHUNK_UPDATE_TIME};
use bevy::app::{App, Plugin};
use bevy::asset::Assets;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventWriter;
use bevy::ecs::system::Commands;
use bevy::math::{IVec2, Vec3};
use bevy::pbr::PbrBundle;
use bevy::prelude::*;
use bevy::prelude::{shape, Mesh, Query, Res, ResMut, StandardMaterial, Transform};
use bevy::prelude::{Color, IntoSystem};
use bevy::reflect::List;
use bevy::utils::{HashMap, Instant};
use building_blocks::core::{Extent3i, PointN};
use building_blocks::prelude::Array3x1;
use building_blocks::prelude::FillExtent;
use ndarray::Array3;
use noise::{NoiseFn, OpenSimplex};
use std::collections::VecDeque;
use bevy::app::CoreStage::Update;
use bevy::diagnostic::{Diagnostic, Diagnostics};

const VOXEL_SIZE: f32 = 0.25;
const STEP_SIZE: f32 = 1.0 / VOXEL_SIZE;
const NOISE_SCALE: f32 = 16.0 / VOXEL_SIZE;

pub type ChunkMap = HashMap<IVec2, Entity>;

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, StageLabel)]
pub enum WorldUpdateStage {
  Update,
  PostUpdate,
  Cleanup,
}

#[inline]
pub fn chunk_extent() -> Extent3i {
  return Extent3i::from_min_and_shape(
    PointN([0; 3]),
    PointN([CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z]),
  );
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Voxel {
  pub attributes: [u8; 4],
}

struct ChunkSpawnRequest(IVec2);
struct ChunkDespawnRequest(IVec2, Entity);
#[derive(Debug)]
struct ChunkLoadRequest(Entity);

pub struct ChunkReadyEvent(pub IVec2, pub Entity);

#[derive(Default)]
pub struct VoxelWorld {
  pub loaded_chunks: ChunkMap,
}

#[derive(Component, Debug)]
pub enum ChunkLoadState {
  Load,
  Unload,
  Despawn,
  Generate,
  Done,
}

#[derive(Component)]
pub struct Chunk {
  pub pos: IVec2,
  pub block_data: Array3x1<Voxel>,
}

#[derive(Bundle)]
pub struct ChunkDataBundle {
  pub transform: Transform,
  pub global_transform: GlobalTransform,
  pub chunk: Chunk,
}

fn update_visible_chunks(
  player_query: Query<(&Player, &Transform)>,
  player_config: Res<PlayerConfig>,
  world: Res<VoxelWorld>,
  mut spawn_requests: EventWriter<ChunkSpawnRequest>,
  mut despawn_requests: EventWriter<ChunkDespawnRequest>,
  mut diagnostics: ResMut<Diagnostics>,
) {
  for (player, transform) in player_query.iter() {

    let start_time = Instant::now();
    let current_chunk_pos = get_chunk_indices(transform.translation);

    let mut load_radius_chunks: Vec<IVec2> = Vec::new();
    let max_distance = player_config.chunk_render_distance as i32;

    for dx in -max_distance..=max_distance {
      for dy in -max_distance..=max_distance {
        // Skip chunks out of render_distance radius
        if dx.pow(2) + dy.pow(2) >= max_distance.pow(2) {
          continue;
        }

        let chunk_pos = current_chunk_pos + IVec2::new(dx, dy);
        if !world.loaded_chunks.contains_key(&chunk_pos) {
          load_radius_chunks.push(chunk_pos);
        }
      }
    }

    load_radius_chunks.sort_by_key(|a| -(a.x.pow(2) + a.y.pow(2)));
    // println!("load radius chunks: {:?}", load_radius_chunks);

    spawn_requests.send_batch(
      load_radius_chunks
        .into_iter()
        .map(|c| ChunkSpawnRequest(c.clone())),
    );

    for key in world.loaded_chunks.keys() {
      let delta = *key - current_chunk_pos;
      let entity = world.loaded_chunks.get(key).unwrap().clone();
      if delta.x.pow(2) + delta.y.pow(2) > player_config.chunk_render_distance.pow(2).into() {
        despawn_requests.send(ChunkDespawnRequest(key.clone(), entity));
      }
    }

    let end_time = Instant::now();
    diagnostics.add_measurement(VISIBLE_CHUNK_UPDATE_TIME, (end_time - start_time).as_secs_f64())
  }
}

fn prepare_for_unload(
  mut despawn_events: EventReader<ChunkDespawnRequest>,
  mut chunks: Query<&mut ChunkLoadState>,
) {
  for despawn_event in despawn_events.iter() {
    if let Ok(mut load_state) = chunks.get_mut(despawn_event.1) {
      *load_state = ChunkLoadState::Unload;
    }
  }
}

fn destroy_chunks(
  mut commands: Commands,
  mut world: ResMut<VoxelWorld>,
  chunks: Query<(&Chunk, &ChunkLoadState)>,
) {
  for (chunk, load_state) in chunks.iter() {
    match load_state {
      ChunkLoadState::Unload => {
        let entity = world.loaded_chunks.remove(&chunk.pos).unwrap();
        commands.entity(entity).despawn();
      }
      _ => {}
    }
  }
}

fn get_chunk_indices(pos: Vec3) -> IVec2 {
  IVec2::new(
    pos.x.floor() as i32 / CHUNK_SIZE_X,
    pos.z.floor() as i32 / CHUNK_SIZE_Z,
  )
}

fn get_global_chunk_coordinates(coords: IVec2) -> Vec3 {
  Vec3::new(
    (coords.x * CHUNK_SIZE_X) as f32,
    0.0,
    (coords.y * CHUNK_SIZE_Z) as f32,
  )
}

fn create_chunks(
  mut commands: Commands,
  mut spawn_events: EventReader<ChunkSpawnRequest>,
  mut world: ResMut<VoxelWorld>,
) {
  for creation_request in spawn_events.iter() {
    let entity = commands
      .spawn_bundle(ChunkDataBundle {
        transform: Transform::from_translation(get_global_chunk_coordinates(creation_request.0)),
        chunk: Chunk {
          pos: creation_request.0,
          block_data: Array3x1::fill(chunk_extent().padded(1), Voxel::default()),
        },
        global_transform: Default::default(),
      })
      .insert(ChunkLoadState::Load)
      .id();

    world.loaded_chunks.insert(creation_request.0, entity);
  }
}

fn load_chunk_data(
  mut chunks: Query<(&mut ChunkLoadState, Entity), Added<Chunk>>,
  mut gen_requests: ResMut<VecDeque<ChunkLoadRequest>>,
  mut diagnostics: ResMut<Diagnostics>
) {
  let start_time = Instant::now();
  for (mut load_state, entity) in chunks.iter_mut() {
    match *load_state {
      ChunkLoadState::Load => {
        *load_state = ChunkLoadState::Generate;
        gen_requests.push_front(ChunkLoadRequest(entity));
      }
      _ => continue,
    }
  }
  let end_time = Instant::now();
  diagnostics.add_measurement(CHUNK_LOAD_DATA_TIME, (end_time - start_time).as_secs_f64())
}

fn generate_chunks(
  player_config: Res<PlayerConfig>,
  mut query: Query<(&mut Chunk, &mut ChunkLoadState)>,
  mut gen_requests: ResMut<VecDeque<ChunkLoadRequest>>,
  mut diagnostics: ResMut<Diagnostics>
) {
  let start_time = Instant::now();
  for _ in 0..MAX_CHUNKS_GEN_PER_FRAME {
    if let Some(ev) = gen_requests.pop_front() {
      if let Ok((mut data, mut load_state)) = query.get_mut(ev.0) {
        generate_chunk(data);
        *load_state = ChunkLoadState::Done;

      }
    }
  }
  let end_time = Instant::now();
  diagnostics.add_measurement(CHUNK_GEN_TIME, (end_time - start_time).as_secs_f64())
}

fn mark_chunks_ready(
  mut ready_events: EventWriter<ChunkReadyEvent>,
  chunks: Query<(&Chunk, &ChunkLoadState, Entity), Changed<ChunkLoadState>>,
) {
  for (chunk, load_state, entity) in chunks.iter() {
    match load_state {
      ChunkLoadState::Done => ready_events.send(ChunkReadyEvent(chunk.pos, entity)),
      _ => {}
    }
  }
}

fn setup_diagnostics(
  mut diagnostics: ResMut<Diagnostics>
) {
  diagnostics.add(Diagnostic::new(VISIBLE_CHUNK_UPDATE_TIME, "Visible chunks calculation time", 1));
  diagnostics.add(Diagnostic::new(CHUNK_LOAD_DATA_TIME, "Chunk load data time", 1));
  diagnostics.add(Diagnostic::new(CHUNK_GEN_TIME, "Chunk generation time", 1));
  diagnostics.add(Diagnostic::new(CHUNK_MESH_TIME, "Chunk meshing time", 1));
}

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
  fn build(&self, app: &mut App) {
    const UPDATE_VISIBLE_CHUNKS_LABEL: &'static str = "update_visible_chunks";
    const CREATE_CHUNKS_LABEL: &'static str = "create_chunks";

    app
      .insert_resource(VoxelWorld::default())
      .init_resource::<VecDeque<ChunkLoadRequest>>()
      .add_event::<ChunkSpawnRequest>()
      .add_event::<ChunkDespawnRequest>()
      .add_event::<ChunkReadyEvent>()
      .add_stage(WorldUpdateStage::Update, SystemStage::parallel())
      .add_stage_after(
        WorldUpdateStage::Update,
        WorldUpdateStage::PostUpdate,
        SystemStage::parallel(),
      )
      .add_stage_after(
        WorldUpdateStage::PostUpdate,
        WorldUpdateStage::Cleanup,
        SystemStage::parallel(),
      )
      .add_system_to_stage(
        WorldUpdateStage::Update,
        update_visible_chunks
          .system()
          .label(UPDATE_VISIBLE_CHUNKS_LABEL),
      )
      .add_system_to_stage(
        WorldUpdateStage::Update,
        create_chunks
          .system()
          .label(CREATE_CHUNKS_LABEL)
          .after(UPDATE_VISIBLE_CHUNKS_LABEL),
      )
      .add_system_to_stage(
        WorldUpdateStage::Update,
        load_chunk_data.system().after(CREATE_CHUNKS_LABEL),
      )
      .add_system_to_stage(WorldUpdateStage::Update, generate_chunks.system())
      .add_system_to_stage(WorldUpdateStage::Update, mark_chunks_ready.system())
      .add_system_to_stage(WorldUpdateStage::Cleanup, prepare_for_unload.system())
      .add_system_to_stage(WorldUpdateStage::Cleanup, destroy_chunks.system())
      .add_startup_system(setup_diagnostics.system());
  }
}
