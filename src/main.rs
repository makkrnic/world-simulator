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

const CAMERA_SPEED: f32 = 1000.0;
const WINDOW_TITLE: &str = "World simulator";

#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

pub struct FlyCam;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: WINDOW_TITLE.to_string(),
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor::default())
        .init_resource::<InputState>()
        .init_resource::<MovementSettings>()
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
        .add_startup_system(setup.system())
        .add_startup_system(initial_grab_cursor.system())
        .add_system(rotate_camera_system.system())
        .add_system(move_camera_system.system())
        .add_system(cursor_grab.system())
        .add_system(update_title.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        material: materials.add(Color::rgb(0.7, 0.4, 0.3).into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.5, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, -8.0, 4.0),
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
        .insert(FlyCam);
}

fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window)
    }
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

fn move_camera_system(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary_mut().unwrap();
    if !window.cursor_locked() {
        return
    }

    for (_camera, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let fwd = -Vec3::new(local_z.x, local_z.y, local_z.z);
        let right = Vec3::new(local_z.z, 0.0, -local_z.x);
        let up = transform.local_y();

        for key in keys.get_pressed() {
            match key {
                KeyCode::W => velocity += fwd,
                KeyCode::S => velocity -= fwd,
                KeyCode::A => velocity -= right,
                KeyCode::D => velocity += right,
                KeyCode::Space => velocity += up,
                KeyCode::LShift => velocity -= up,
                _ => (),
            }
        }

        velocity = velocity.normalize();

        if !velocity.is_nan() {
            transform.translation += velocity * time.delta_seconds() * settings.speed;
        }
    }
}

fn rotate_camera_system(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<(&FlyCam, &mut Transform, )>,
){
    let window = windows.get_primary().unwrap();
    if !(window.cursor_locked() || buttons.pressed(MouseButton::Middle)) {
        return
    }

    for (_camera, mut transform) in query.iter_mut() {
        for ev in state.reader_motion.iter(&motion) {
            let window_scale = window.height().min(window.width());


            state.pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
            state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
        }

        state.pitch = state.pitch.clamp(-1.54, 1.54);

        transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
    }
}
