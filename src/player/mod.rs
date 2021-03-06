use crate::config::{MovementSettings, PlayerConfig};
use crate::world::WORLD_RESOLUTION;
use bevy::{
  app::{EventReader, ManualEventReader, Plugin},
  asset::Assets,
  input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButtonInput, MouseMotion},
    ElementState, Input,
  },
  math::{const_vec3, Quat, Vec3},
  prelude::{
    shape, App, BuildChildren, Bundle, Children, Color, Commands, Component, GlobalTransform,
    IntoSystem, Mesh, MouseButton, PbrBundle, PerspectiveCameraBundle, Query, Res, ResMut,
    StandardMaterial, Time, Transform, Vec2, Windows, With, Without,
  },
  render::camera::PerspectiveProjection,
  utils::HashMap,
};
use building_blocks::core::sdfu::mods::Translate;
use std::ops::Deref;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Action {
  MoveFwd,
  MoveBwd,
  MoveLeft,
  MoveRight,
  MoveUp,
  MoveDown,

  Speedup,

  ToggleCursorGrab,
}

#[derive(Default)]
pub struct CursorGrabStatus(bool);

#[derive(Hash, Eq, PartialEq)]
enum UserInput {
  Keyboard(KeyCode),
  // TODO
  // Mouse(MouseButton),
}

struct KeyBinds(HashMap<UserInput, Action>);

impl Deref for KeyBinds {
  type Target = HashMap<UserInput, Action>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Default for KeyBinds {
  fn default() -> Self {
    let mut binds = HashMap::default();
    binds.insert(UserInput::Keyboard(KeyCode::W), Action::MoveFwd);
    binds.insert(UserInput::Keyboard(KeyCode::S), Action::MoveBwd);
    binds.insert(UserInput::Keyboard(KeyCode::A), Action::MoveLeft);
    binds.insert(UserInput::Keyboard(KeyCode::D), Action::MoveRight);
    binds.insert(UserInput::Keyboard(KeyCode::Space), Action::MoveUp);
    binds.insert(UserInput::Keyboard(KeyCode::LShift), Action::MoveDown);
    binds.insert(UserInput::Keyboard(KeyCode::LControl), Action::Speedup);
    binds.insert(
      UserInput::Keyboard(KeyCode::Escape),
      Action::ToggleCursorGrab,
    );

    KeyBinds(binds)
  }
}

fn handle_user_input(
  mut actions: ResMut<Input<Action>>,
  keybinds: Res<KeyBinds>,
  mut kb_events: EventReader<KeyboardInput>,
  mut mouse_events: EventReader<MouseButtonInput>,
) {
  actions.clear();
  for kb_event in kb_events.iter() {
    if let KeyboardInput {
      key_code: Some(key_code),
      state,
      ..
    } = kb_event
    {
      if let Some(action) = keybinds.get(&UserInput::Keyboard(*key_code)) {
        match state {
          ElementState::Pressed => actions.press(*action),
          ElementState::Released => actions.release(*action),
        }
      }
    }
  }
}

fn handle_player_input(
  mut query: Query<(&mut PlayerController, &mut Transform)>,
  actions: Res<Input<Action>>,
  time: Res<Time>,
  settings: Res<MovementSettings>,
) {
  for (mut controller, mut transform) in query.iter_mut() {
    let mut direction = Vec3::ZERO;
    let fwd = (transform.rotation.mul_vec3(Vec3::Z) * const_vec3!([1.0, 0.0, 1.0])).normalize();
    let right = transform.rotation.mul_vec3(Vec3::X).normalize();
    let mut speedup_factor = 1.0;

    for action in actions.get_pressed() {
      match action {
        Action::MoveFwd => direction.z -= 1.0,
        Action::MoveBwd => direction.z += 1.0,
        Action::MoveRight => direction.x += 1.0,
        Action::MoveLeft => direction.x -= 1.0,
        Action::MoveUp => direction.y += 1.0,
        Action::MoveDown => direction.y -= 1.0,
        Action::Speedup => speedup_factor = 10.0,
        Action::ToggleCursorGrab => {
          if actions.just_pressed(*action) {
            controller.cursor_grab = !controller.cursor_grab
          }
        }
      }
    }

    if direction == Vec3::ZERO {
      break;
    }

    direction = direction
      * speedup_factor
      * settings.speed
      * time.delta_seconds()
      * (WORLD_RESOLUTION as f32);

    transform.translation += (direction.x * right + direction.z * fwd + direction.y * Vec3::Y);
  }
}

fn handle_mouse_move(
  settings: Res<MovementSettings>,
  player_config: Res<PlayerConfig>,
  mut q_player: Query<(&mut PlayerController, &mut Transform, &Children), Without<PlayerCamera>>,
  mut q_player_camera: Query<(&PlayerCamera, &mut Transform)>,
  mut mouse_motion_reader: EventReader<MouseMotion>,
  windows: Res<Windows>,
) {
  let window = windows.get_primary().unwrap();
  if !window.cursor_locked() {
    return;
  }

  for (mut controller, mut transform, children) in q_player.iter_mut() {
    println!("Player: {:?}", transform);
    println!("Children: {:?}", children);
    let mut camera_transform_entity = None;
    let mut player_camera_transform = None;
    for &child in children.iter() {
      if let Ok(_) = q_player_camera.get_mut(child) {
        camera_transform_entity = Some(child);
        break;
      }
    }

    if camera_transform_entity == None {
      continue;
    }
    if let Ok((_, mut pc_transform)) = q_player_camera.get_mut(camera_transform_entity.unwrap()) {
      player_camera_transform = Some(pc_transform);
    }

    let mut movement = Vec2::ZERO;
    for mouse_move in mouse_motion_reader.iter() {
      movement += mouse_move.delta;
    }

    let window_scale = window.height().min(window.width());

    controller.pitch -= (settings.sensitivity * movement.y * window_scale).to_radians();
    controller.pitch = controller.pitch.clamp(-1.54, 1.54);
    controller.yaw -= (settings.sensitivity * movement.x * window_scale).to_radians();

    transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw);
    if let Some(mut pc_transform) = player_camera_transform {
      pc_transform.rotation = Quat::from_axis_angle(Vec3::X, controller.pitch);
      pc_transform.translation = Vec3::new(
        0.0,
        -controller.pitch.sin() * player_config.camera_distance,
        controller.pitch.cos() * player_config.camera_distance,
      );
    }
  }
}

#[derive(Default)]
struct PlayerInputState {
  reader_motion: ManualEventReader<MouseMotion>,
  pitch: f32,
  yaw: f32,
}

#[derive(Component, Debug)]
pub struct PlayerCamera; // {
                         // /// Head position
                         // pos: Vec3,

// /// For now, our player will be a box
// bounding_box: Vec3,
// }

// impl Default for Player {
//   fn default() -> Self {
//     Self {
//       bounding_box: Vec3::new(2.0, 4.0, 1.5),
//       ..Default::default()
//     }
//   }
// }

fn update_cursor_grab(mut windows: ResMut<Windows>, query: Query<&PlayerController>) {
  let mut window = windows.get_primary_mut().unwrap();
  for controller in query.get_single() {
    window.set_cursor_lock_mode(controller.cursor_grab);
    window.set_cursor_visibility(!controller.cursor_grab);
  }
}

fn setup_player_camera(
  player_config: Res<PlayerConfig>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands
    .spawn_bundle(Player {
      transform: Transform::from_xyz(-20.0, 200.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
      ..Default::default()
    })
    .with_children(|parent| {
      let default_controller = PlayerController::default();
      let camera_rot = Quat::from_axis_angle(Vec3::X, default_controller.pitch);
      let camera_offset = Vec3::new(
        0.0,
        -default_controller.pitch.sin() * player_config.camera_distance,
        default_controller.pitch.cos() * player_config.camera_distance,
      );

      parent
        .spawn_bundle(PerspectiveCameraBundle {
          transform: Transform {
            rotation: camera_rot,
            translation: camera_offset,
            ..Default::default()
          },
          perspective_projection: PerspectiveProjection {
            near: 0.1,
            ..Default::default()
          },
          ..Default::default()
        })
        .insert(PlayerCamera);

      parent.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 2.0, 0.5))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
      });
    })
    .insert(Player::default())
    .insert(PlayerController::default());
}

#[derive(Debug, Default, Component, Bundle)]
pub struct Player {
  transform: Transform,
  global_transform: GlobalTransform,
}

#[derive(Component, Debug)]
pub struct PlayerController {
  pub cursor_grab: bool,
  pub pitch: f32,
  pub yaw: f32,
}

impl Default for PlayerController {
  fn default() -> Self {
    Self {
      pitch: -0.8,
      yaw: 0.0,
      cursor_grab: false,
    }
  }
}

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<PlayerController>()
      .init_resource::<KeyBinds>()
      .init_resource::<Input<Action>>()
      .init_resource::<PlayerInputState>()
      .init_resource::<CursorGrabStatus>()
      .init_resource::<MovementSettings>()
      .add_system(handle_user_input.system())
      .add_system(handle_player_input.system())
      .add_system(handle_mouse_move.system())
      .add_system(update_cursor_grab.system())
      .add_startup_system(setup_player_camera.system());
  }
}
