mod fly_camera;
mod world;
mod config;

use bevy::app::{Events, ManualEventReader};
use bevy::prelude::*;

use bevy::asset::AssetPlugin;
// use bevy::audio::AudioPlugin;
use bevy::core::CorePlugin;
use bevy::diagnostic::{Diagnostic, DiagnosticId, Diagnostics, DiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::gltf::GltfPlugin;
use bevy::input::InputPlugin;
use bevy::log::LogPlugin;
use bevy::pbr::PbrPlugin;
use bevy::render::RenderPlugin;
use bevy::scene::ScenePlugin;
use bevy::sprite::SpritePlugin;
use bevy::text::TextPlugin;
use bevy::ui::UiPlugin;
use bevy::wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions, WgpuPlugin};
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;

use crate::fly_camera::{FlyCam, FlyCamPlugin};
use crate::world::VoxelWorldPlugin;
use bevy::input::mouse::MouseMotion;
use bevy::render::pass::ClearColor;
use bevy::render::wireframe::{WireframeConfig, WireframePlugin};
use bevy::wgpu::WgpuBackend::Vulkan;
use crate::config::PlayerConfig;

const WINDOW_TITLE: &str = "World simulator";

struct FPSCounter(Timer, u32);

#[derive(Default)]
struct State {
    fps: u32,
}

fn main() {
    App::new()
        .insert_resource(PlayerConfig::default())
        .insert_resource(WindowDescriptor {
            title: WINDOW_TITLE.to_string(),
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor::default())
        .insert_resource(FPSCounter(Timer::from_seconds(1.0, true), 0))
        // .insert_resource(Msaa { samples: 2 })
        .insert_resource(WgpuOptions {
            backend: Vulkan,
            features: WgpuFeatures {
                features: vec![WgpuFeature::NonFillPolygonMode]
            },
            ..Default::default()
        })
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
        .add_plugin(WireframePlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup_diagnostic_system.system())
        .add_startup_system(setup.system())
        .add_system(update_title.system())
        .add_plugin(VoxelWorldPlugin)
        .add_system(fps_counter.system())
        .init_resource::<State>()
        .run();
}

pub const DIAGNOSTIC_FPS: DiagnosticId = DiagnosticId::from_u128(1);

fn setup_diagnostic_system(mut diagnostics: ResMut<Diagnostics>) {
    diagnostics.add(Diagnostic::new(
        DIAGNOSTIC_FPS,
        "fps",
        100
    ))
}

#[derive(Debug, Bundle, Default)]
struct DirLightBundle {
    pub dir_light: DirectionalLight,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn_bundle(DirLightBundle {
        dir_light: DirectionalLight::new(Color::WHITE, 100000.0, Vec3::new(-2.0, -1.0, -3.0)),
        ..Default::default()
    });
}

fn update_title(
    mut windows: ResMut<Windows>,
    mut query: Query<(&FlyCam, &mut Transform)>,
    state: Res<State>,
) {
    let window = windows.get_primary_mut().unwrap();

    let mut position_title = "".to_string();
    for (_camera, transform) in query.iter_mut() {
        let local_z = transform.local_z();
        position_title = format!(
            "position: {:?}, Local z: {:?}",
            transform.translation, local_z
        )
        .to_string();
    }

    let mut locked_title = WINDOW_TITLE.to_owned();

    if window.cursor_locked() {
        locked_title = locked_title + " - Press ESC to release cursor";
    }

    let fps = format!("{} FPS ", state.fps);

    window.set_title(fps.to_string() + &locked_title.to_string() + " " + &position_title);
}

fn fps_counter(time: Res<Time>, mut timer: ResMut<FPSCounter>, mut state: ResMut<State>, mut diagnostics: ResMut<Diagnostics>) {
    timer.0.tick(time.delta());
    timer.1 += 1;
    if timer.0.finished() {
        state.fps = timer.1;
        diagnostics.add_measurement(DIAGNOSTIC_FPS, state.fps as f64);
        timer.1 = 0;
    }
}
