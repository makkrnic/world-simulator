use bevy::prelude::{Color, IntoSystem};
use bevy::app::{AppBuilder, Plugin};
use bevy::asset::Assets;
use bevy::ecs::system::Commands;
use bevy::math::Vec3;
use bevy::pbr::PbrBundle;
use bevy::prelude::{Mesh, ResMut, shape, StandardMaterial, Transform};
use noise::{NoiseFn, Perlin};

const CHUNK_SIZE_X: i64 = 32;
const CHUNK_SIZE_Y: i64 = 32;
const VOXEL_SIZE: f32 = 0.5;
const STEP_SIZE: f32 = 1.0 / VOXEL_SIZE;
const NOISE_SCALE: f64 = 128.0;

fn generate_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let p = Perlin::new();
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: VOXEL_SIZE as f32 }));
    let cube_mat = materials.add(Color::rgb(0.7, 0.4, 0.3).into());

    for y in 0..CHUNK_SIZE_Y {
        for x in 1..CHUNK_SIZE_X {
            let mut height = (p.get([x as f64 / NOISE_SCALE, y as f64 / NOISE_SCALE]) * 16.0) as f32;
            height = (height * STEP_SIZE).trunc() / STEP_SIZE;

            commands.spawn_bundle(PbrBundle {
                mesh: cube_mesh.clone(),
                material: cube_mat.clone(),
                transform: Transform {
                    translation: Vec3::new(x as f32 / STEP_SIZE , height as f32, y as f32 / STEP_SIZE),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_startup_system(generate_world.system())
        ;
    }
}