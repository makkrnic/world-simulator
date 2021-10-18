mod fly_camera;
mod world;

use bevy::app::{Events, ManualEventReader};
use bevy::prelude::*;

use bevy::asset::AssetPlugin;
// use bevy::audio::AudioPlugin;
use bevy::core::CorePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::gltf::GltfPlugin;
use bevy::input::InputPlugin;
use bevy::log::LogPlugin;
use bevy::pbr::PbrPlugin;
use bevy::render::RenderPlugin;
use bevy::scene::ScenePlugin;
use bevy::sprite::SpritePlugin;
use bevy::text::TextPlugin;
use bevy::ui::UiPlugin;
use bevy::wgpu::WgpuPlugin;
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;

use bevy::render::pass::ClearColor;
use bevy::input::mouse::MouseMotion;
use crate::fly_camera::{FlyCam, FlyCamPlugin};
use crate::world::WorldPlugin;

const WINDOW_TITLE: &str = "World simulator";

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: WINDOW_TITLE.to_string(),
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor::default())
        .add_plugin(LogPlugin)
        .add_plugin(CorePlugin)
        .add_plugin(TransformPlugin)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(WindowPlugin::default())
        .add_plugin(AssetPlugin)
        .add_plugin(ScenePlugin)
        .add_plugin(RenderPlugin::default())
        .add_plugin(SpritePlugin)
        .add_plugin(PbrPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(TextPlugin)
        // .add_plugin(AudioPlugin)
        .add_plugin(GilrsPlugin)
        .add_plugin(GltfPlugin)
        .add_plugin(WinitPlugin)
        .add_plugin(WgpuPlugin)
        .add_plugin(FlyCamPlugin)
        .add_startup_system(setup.system())
        .add_system(update_title.system())
        .add_plugin(WorldPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, -8.0, 4.0),
        ..Default::default()
    });
}

fn update_title(
    mut windows: ResMut<Windows>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary_mut().unwrap();

    let mut position_title = "".to_string();
    for (_camera, transform) in query.iter_mut() {
        let local_z = transform.local_z();
        position_title = format!("position: {:?}, Local z: {:?}", transform.translation, local_z).to_string();
    }

    let mut locked_title = WINDOW_TITLE.to_owned();

    if window.cursor_locked() {
        locked_title = locked_title + " - Press ESC to release cursor";
    }

    window.set_title(locked_title.to_string() + " " + &position_title);
}
