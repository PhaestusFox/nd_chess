use bevy::prelude::*;
use bevy_granite::{bevy_granite_editor::BevyGraniteEditor, prelude::*};

mod board;

mod camera;

mod pieces;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins((board::BoardPlugin, camera::CameraPlugin));
    app.add_plugins(pieces::PiecesPlugin);
    app.add_plugins(bevy::picking::mesh_picking::MeshPickingPlugin);
    app.run();
}
