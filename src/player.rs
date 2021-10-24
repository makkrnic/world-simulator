use bevy::app::{App, ManualEventReader, Plugin};
use bevy::input::Input;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseMotion;
use bevy::math::Vec3;
use bevy::prelude::MouseButton;
use bevy::utils::HashMap;

#[derive(Debug, Hash, Eq, PartialEq)]
enum Action {
  MoveFwd,
  MoveBwd,
  MoveLeft,
  MoveRight,
  MoveUp,
  MoveDown,

  Speedup,
}

#[derive(Hash, Eq, PartialEq)]
enum UserInput {
  Keyboard(KeyCode),
  Mouse(MouseButton),
}

struct KeyBinds(HashMap<UserInput, Action>);

impl Default for KeyBinds {
  fn default() -> Self {
    let mut binds = HashMap::default();
    binds.insert(UserInput::Keyboard(KeyCode::W), Action::MoveFwd);
    binds.insert(UserInput::Keyboard(KeyCode::A), Action::MoveBwd);
    binds.insert(UserInput::Keyboard(KeyCode::S), Action::MoveLeft);
    binds.insert(UserInput::Keyboard(KeyCode::D), Action::MoveRight);
    binds.insert(UserInput::Keyboard(KeyCode::LShift), Action::MoveUp);
    binds.insert(UserInput::Keyboard(KeyCode::Space), Action::MoveDown);
    binds.insert(UserInput::Keyboard(KeyCode::Capital), Action::Speedup);

    KeyBinds(binds)
  }
}


#[derive(Default)]
struct PlayerInputState {
  reader_motion: ManualEventReader<MouseMotion>,
  pitch: f32,
  yaw: f32,
}

struct Player {
  /// Head position
  pos: Vec3,

  /// For now, our player will be a box
  bounding_box: Vec3,
}

impl Default for Player {
  fn default() -> Self {
    Self {
      bounding_box: Vec3::new(2.0, 4.0, 1.5),
      ..Default::default()
    }
  }
}

struct PlayerController {
  player: Player,
}

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<Input<Action>>()
      .init_resource::<PlayerInputState>();
  }
}