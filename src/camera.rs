use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_flycam::NoCameraPlayerPlugin);
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (update_camera_view, move_camera));
    }
}

#[derive(Component)]
struct BoardCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        BoardCamera,
        Camera3d::default(),
        BoardCameraView::new(Value::Zero, Value::Zero, Value::Neg),
        bevy_flycam::FlyCam,
    ));
}

#[derive(Component, Debug)]
struct BoardCameraView {
    x: Value,
    z: Value,
    y: Value,
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Pos,
    Zero,
    Neg,
}

impl Value {
    fn offset(&self) -> f32 {
        match self {
            Value::Pos => 16.0,
            Value::Zero => 0.0,
            Value::Neg => -16.0,
        }
    }
}

impl BoardCameraView {
    fn new(x: Value, y: Value, z: Value) -> Self {
        Self { x, y, z }
    }

    fn transform(&self) -> Transform {
        Transform::from_translation(
            Vec3::new(self.x.offset(), self.y.offset(), self.z.offset()) + Vec3::splat(4.),
        )
        .looking_at(Vec3::splat(4.), Vec3::Y)
    }

    fn left(&mut self) {
        match self {
            BoardCameraView {
                x: Value::Pos,
                z: Value::Zero,
                y: _,
            } => {
                self.z = Value::Neg;
            }
            BoardCameraView {
                x: Value::Pos,
                z: Value::Neg,
                y: _,
            } => {
                self.x = Value::Zero;
            }
            BoardCameraView {
                x: Value::Zero,
                z: Value::Neg,
                y: _,
            } => {
                self.x = Value::Neg;
            }
            BoardCameraView {
                x: Value::Neg,
                z: Value::Neg,
                y: _,
            } => {
                self.z = Value::Zero;
            }
            BoardCameraView {
                x: Value::Neg,
                z: Value::Zero,
                y: _,
            } => {
                self.z = Value::Pos;
            }
            BoardCameraView {
                x: Value::Neg,
                z: Value::Pos,
                y: _,
            } => {
                self.x = Value::Zero;
            }
            BoardCameraView {
                x: Value::Zero,
                z: Value::Pos,
                y: _,
            } => {
                self.x = Value::Pos;
            }
            BoardCameraView {
                x: Value::Pos,
                z: Value::Pos,
                y: _,
            } => {
                self.z = Value::Zero;
            }
            BoardCameraView {
                x: Value::Zero,
                z: Value::Zero,
                y: _,
            } => {}
        }
    }

    fn right(&mut self) {
        match self {
            BoardCameraView {
                x: Value::Pos,
                z: Value::Zero,
                y: _,
            } => {
                self.z = Value::Pos;
            }
            BoardCameraView {
                x: Value::Pos,
                z: Value::Pos,
                y: _,
            } => {
                self.x = Value::Zero;
            }
            BoardCameraView {
                x: Value::Zero,
                z: Value::Pos,
                y: _,
            } => {
                self.x = Value::Neg;
            }
            BoardCameraView {
                x: Value::Neg,
                z: Value::Pos,
                y: _,
            } => {
                self.z = Value::Zero;
            }
            BoardCameraView {
                x: Value::Neg,
                z: Value::Zero,
                y: _,
            } => {
                self.z = Value::Neg;
            }
            BoardCameraView {
                x: Value::Neg,
                z: Value::Neg,
                y: _,
            } => {
                self.x = Value::Zero;
            }
            BoardCameraView {
                x: Value::Zero,
                z: Value::Neg,
                y: _,
            } => {
                self.x = Value::Pos;
            }
            BoardCameraView {
                x: Value::Pos,
                z: Value::Neg,
                y: _,
            } => {
                self.z = Value::Zero;
            }
            BoardCameraView {
                x: Value::Zero,
                z: Value::Zero,
                y: _,
            } => {}
        }
    }

    fn up(&mut self) {
        match self {
            BoardCameraView {
                y: Value::Zero,
                x: _,
                z: _,
            } => {
                self.y = Value::Pos;
            }
            BoardCameraView {
                y: Value::Neg,
                x: _,
                z: _,
            } => {
                self.y = Value::Zero;
            }
            _ => {}
        }
    }

    fn down(&mut self) {
        match self {
            BoardCameraView {
                y: Value::Zero,
                x: _,
                z: _,
            } => {
                self.y = Value::Neg;
            }
            BoardCameraView {
                y: Value::Pos,
                x: _,
                z: _,
            } => {
                self.y = Value::Zero;
            }
            _ => {}
        }
    }
}

fn update_camera_view(
    mut camera: Populated<(&mut Transform, &BoardCameraView), Changed<BoardCameraView>>,
) {
    for (mut transform, view) in camera.iter_mut() {
        *transform = view.transform();
    }
}

fn move_camera(
    mut camera: Single<&mut BoardCameraView, With<BoardCamera>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.any_just_pressed([KeyCode::ArrowLeft]) {
        camera.left();
    }
    if input.any_just_pressed([KeyCode::ArrowRight]) {
        camera.right();
    }
    if input.any_just_pressed([KeyCode::ArrowUp]) {
        camera.up();
    }
    if input.any_just_pressed([KeyCode::ArrowDown]) {
        camera.down();
    }
}
