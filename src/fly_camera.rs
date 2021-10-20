use bevy::app::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct FlyCam;

const CAMERA_SPEED: f32 = 12.0;
const SPEEDUP_KEY: KeyCode = KeyCode::LControl;

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
            speed: CAMERA_SPEED,
        }
    }
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
        return;
    }

    for (_camera, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let fwd = -Vec3::new(local_z.x, local_z.y, local_z.z);
        let right = Vec3::new(local_z.z, 0.0, -local_z.x);
        let up = transform.local_y();

        let mut speed_factor = 1.0;
        for key in keys.get_pressed() {
            match key {
                KeyCode::W => velocity += fwd,
                KeyCode::S => velocity -= fwd,
                KeyCode::A => velocity -= right,
                KeyCode::D => velocity += right,
                KeyCode::Space => velocity += up,
                KeyCode::LShift => velocity -= up,
                &SPEEDUP_KEY => speed_factor = 10.0,
                _ => (),
            }
        }

        velocity = velocity.normalize();

        if !velocity.is_nan() {
            transform.translation += velocity * time.delta_seconds() * settings.speed * speed_factor;
        }
    }
}

fn rotate_camera_system(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    if !(window.cursor_locked() || buttons.pressed(MouseButton::Middle)) {
        return;
    }

    for (_camera, mut transform) in query.iter_mut() {
        for ev in state.reader_motion.iter(&motion) {
            let window_scale = window.height().min(window.width());

            state.pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
            state.yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
        }

        state.pitch = state.pitch.clamp(-1.54, 1.54);

        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, state.yaw) * Quat::from_axis_angle(Vec3::X, state.pitch);
    }
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

fn setup(mut commands: Commands) {
    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 200.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);
}

pub struct FlyCamPlugin;

impl Plugin for FlyCamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(initial_grab_cursor.system())
            .add_system(rotate_camera_system.system())
            .add_system(move_camera_system.system())
            .add_system(cursor_grab.system())
            .add_startup_system(setup.system());
    }
}
