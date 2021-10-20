use std::collections::VecDeque;
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
    }
};
use bevy::pbr::render_graph::PBR_PIPELINE_HANDLE;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use bevy::render::render_graph::base::MainPass;
use bevy::render::shader::ShaderStages;
use building_blocks::{
    prelude::*,
    mesh::{
        greedy_quads,
        GreedyQuadsBuffer,
        RIGHT_HANDED_Y_UP_CONFIG,
        OrientedCubeFace,
        UnorientedQuad,
    }
};
use building_blocks::mesh::{IsOpaque, MergeVoxel};
use building_blocks::prelude::IsEmpty;
use crate::config::PlayerConfig;
use crate::world::{Chunk, chunk_extent, ChunkReadyEvent, Voxel};

struct ChunkMeshingEvent(Entity);

pub const TERRAIN_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 1709742349218822681);

#[derive(Bundle)]
pub struct ChunkRenderBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
}

fn attach_chunk_render_bundle(
    chunks: Query<Entity, Added<Chunk>>,
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for ent in chunks.iter() {
        commands.entity(ent).insert_bundle(ChunkRenderBundle {
            mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
            material: mats.add(Default::default()),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                PBR_PIPELINE_HANDLE.typed(),
                // TERRAIN_PIPELINE_HANDLE.typed(),
            )]),
            draw: Default::default(),
            main_pass: Default::default(),
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
        });
    }
}

fn mesh_chunks_async(
    player_config: Res<PlayerConfig>,
    mut chunks: Query<(&Chunk, &mut Visible, &Handle<Mesh>)>,
    mut meshing_events: ResMut<VecDeque<ChunkMeshingEvent>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in 0..(player_config.chunk_render_distance / 2) {
        if let Some(meshing_event) = meshing_events.pop_back() {
            if let Ok((chunk, mut visibility, mesh_handle)) = chunks.get_mut(meshing_event.0) {
                let mesh = meshes.get_mut(mesh_handle).unwrap();
                let extent = chunk_extent();
                let mut greedy_buffer = GreedyQuadsBuffer::new(extent.padded(1), RIGHT_HANDED_Y_UP_CONFIG.quad_groups());
                greedy_quads(&chunk.block_data, &extent.padded(1), &mut greedy_buffer);

                let mut chunk_mesh = ChunkMesh::default();

                for group in greedy_buffer.quad_groups.iter() {
                    for quad in group.quads.iter() {
                        chunk_mesh.add_quad_to_mesh(
                            &group.face,
                            quad,
                            &chunk.block_data.get(quad.minimum)
                        );
                    }
                }

                let ChunkMesh {
                    positions,
                    normals,
                    indices,
                    colors,
                    uv
                } = chunk_mesh;

                mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                mesh.set_indices(Some(Indices::U32(indices)));

                visibility.is_visible = true;
            }
        }
    }
}

fn handle_chunk_ready_events(
    mut ready_events: EventReader<ChunkReadyEvent>,
    mut meshing_events: ResMut<VecDeque<ChunkMeshingEvent>>,
) {
    for ready_event in ready_events.iter() {
        meshing_events.push_front(ChunkMeshingEvent(ready_event.1));
    }
}

fn setup_render_resources(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    asset_server: Res<AssetServer>,
) {
    // pipelines.set_untracked(
    //     TERRAIN_PIPELINE_HANDLE,
    //     PipelineDescriptor::default_config(ShaderStages {})
    // )
}

pub struct WorldRenderPlugin;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ChunkMeshingEvent>()
            .init_resource::<VecDeque<ChunkMeshingEvent>>()
            .add_startup_system(setup_render_resources.system())
            .add_system(attach_chunk_render_bundle.system())
            .add_system(handle_chunk_ready_events.system())
            .add_system(mesh_chunks_async.system())
        ;
    }
}

impl MergeVoxel for Voxel {
    type VoxelValue = u8;

    fn voxel_merge_value(&self) -> Self::VoxelValue {
        self.attributes[0]
    }
}

impl IsOpaque for Voxel {
    fn is_opaque(&self) -> bool {
        true
    }
}

impl IsEmpty for Voxel {
    fn is_empty(&self) -> bool {
        self.attributes[3] == 0
    }
}

#[derive(Default)]
struct ChunkMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uv: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,
}

impl ChunkMesh {
    fn add_quad_to_mesh(&mut self, face: &OrientedCubeFace, quad: &UnorientedQuad, voxel: &Voxel) {
        let start_index = self.positions.len() as u32;

        self.positions.extend_from_slice(&face.quad_mesh_positions(quad, 1.0));
        self.normals.extend_from_slice(&face.quad_mesh_normals());
        self.uv.extend_from_slice(&face.tex_coords(RIGHT_HANDED_Y_UP_CONFIG.u_flip_face, false, quad));
        self.colors.extend_from_slice(&[voxel.attributes; 4]);
        self.indices.extend_from_slice(&face.quad_mesh_indices(start_index));
    }
}