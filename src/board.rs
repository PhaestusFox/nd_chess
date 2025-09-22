use bevy::{
    asset::RenderAssetUsages, ecs::world::DeferredWorld, prelude::*,
    render::render_resource::TextureFormat,
};

use crate::{board::spawner::DimensionSpawner, pieces::Team};

mod spawner;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardResource>()
            .init_resource::<BoardState>()
            .insert_resource(Dimensions(4))
            .add_systems(Startup, spawn_board)
            .register_type::<Index>()
            .register_type::<Dimensions>()
            .register_type::<DimensionSpawner>();
        app.add_systems(Last, captured_piece);
    }
}

#[derive(Resource)]
struct BoardResource {
    cube_handle: Handle<Mesh>,
    black_material: Handle<StandardMaterial>,
    white_material: Handle<StandardMaterial>,
}

impl FromWorld for BoardResource {
    fn from_world(world: &mut World) -> Self {
        // let cube_handle = world
        //     .resource_mut::<Assets<Mesh>>()
        //     .add(Mesh::from(Cuboid::from_size(Vec3::new(1.0, 0.01, 8.0))));
        let cube_handle = world
            .resource_mut::<Assets<Mesh>>()
            .add(get_rectangle_mesh());
        let mut images = world.resource_mut::<Assets<Image>>();
        let pixel = 1.0f32.to_be_bytes().repeat(4);
        let mut w_image = Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 1,
                height: 8,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &pixel,
            TextureFormat::Rgba32Float,
            bevy::asset::RenderAssetUsages::all(),
        );
        let mut b_image = Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 1,
                height: 8,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &pixel,
            TextureFormat::Rgba32Float,
            bevy::asset::RenderAssetUsages::all(),
        );
        for i in (0..8).step_by(2) {
            let _ = b_image.set_color_at(0, i, Color::WHITE);
            let _ = w_image.set_color_at(0, i + 1, Color::WHITE);
        }
        let w_handle = images.add(w_image);
        let b_handle = images.add(b_image);
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let black_material = materials.add(StandardMaterial {
            base_color_texture: Some(b_handle),
            ..Default::default()
        });
        let white_material = materials.add(StandardMaterial {
            base_color_texture: Some(w_handle),
            ..Default::default()
        });

        BoardResource {
            cube_handle,
            black_material,
            white_material,
        }
    }
}

fn get_rectangle_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(),
    );
    // ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [1.0, 0.0]),
    // ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [0.0, 0.0]),
    // ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [0.0, 1.0]),
    // ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [1.0, 1.0]),
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            // Front
            [0.5, 0.0, -7.5],
            [-0.5, 0.0, -7.5],
            [-0.5, 0.0, 0.5],
            [0.5, 0.0, 0.5],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 1.0, 0.0]; 4], // All normals point up
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
    );
    mesh.insert_indices(bevy::render::mesh::Indices::U32(vec![
        0, 1, 2, // First triangle
        2, 3, 0, // Second triangle
        2, 1, 0, // Back face
        0, 3, 2, // Back face
    ]));
    mesh
}

#[derive(Component, Deref, Reflect)]
struct Index(u8);

#[derive(Deref, Clone, Component, Hash, PartialEq, Eq, Default, Debug)]
pub struct Position(pub Vec<u8>);

impl Position {
    fn new(world: &DeferredWorld, cell: Entity) -> Self {
        let mut out = Position(vec![0]);
        let mut next = cell;
        while let Some(parent) = world.get::<ChildOf>(next) {
            let Some(index) = world.get::<Index>(next) else {
                break;
            };
            out.0.push(index.0);
            next = parent.parent();
        }
        out
    }
    fn sum(&self) -> usize {
        self.0.iter().map(|&x| x as usize).sum()
    }
    fn add_dimension(&mut self, dimension: usize, index: u8) {
        let len = self.len();
        if len < dimension {
            self.0.extend((0..(dimension - len)).map(|_| 0));
        }
        self.0[dimension] = index;
    }
    pub fn inc(mut self, dimension: usize) -> Self {
        if let Some(last) = self.0.get_mut(dimension) {
            *last += 1;
        }
        self
    }

    pub fn dec(mut self, dimension: usize) -> Self {
        if let Some(last) = self.0.get_mut(dimension) {
            *last -= 1;
        }
        self
    }

    pub fn last(&self) -> u8 {
        self.0.last().copied().unwrap_or(8)
    }

    pub fn to_translation(&self) -> Vec3 {
        let mut pos = Vec3::ZERO;
        for (d, i) in self.iter().enumerate() {
            if let Some(step) = spawner::render_dimension_step_size(d + 1) {
                pos += step * (*i as f32);
            }
        }
        pos
    }

    /// currently only dimensions < 7 are visible
    /// in future will take in a map of rendered dimensions and return true if the piece would be visible with the currently rendered dimensions
    pub fn is_visible(&self, dimensions: usize) -> bool {
        dimensions < 7
    }
}

impl core::ops::Add for Position {
    type Output = Position;

    fn add(mut self, rhs: Self) -> Self::Output {
        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a += *b;
        }
        self
    }
}

pub struct DimensionIter<'a> {
    pos: &'a Position,
    dimension: usize,
    range: std::ops::Range<i8>,
}

impl DimensionIter<'_> {
    pub fn new(
        pos: &Position,
        dimension: usize,
        range: std::ops::Range<i8>,
    ) -> Option<DimensionIter> {
        let Some(dim) = pos.0.get(dimension) else {
            error!("Position does not have dimension {dimension}");
            return None;
        };
        let mut min = range.start;
        if min < 0 && min.unsigned_abs() > *dim {
            min = -(*dim as i8);
        }
        let mut max = range.end;
        if max > 0 && max as u8 + dim > 8 {
            max = 7 - *dim as i8;
        }
        Some(DimensionIter {
            pos,
            dimension,
            range: min..max,
        })
    }
}

impl Iterator for DimensionIter<'_> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let &i = self.pos.get(self.dimension)?;
        let offset = self.range.next()?;
        let delta = offset.unsigned_abs();
        let new = if offset < 0 { i - delta } else { i + delta };
        let mut new_pos = self.pos.clone();
        new_pos.0[self.dimension] = new;
        Some(new_pos)
    }
}

pub struct PositionIter<'a> {
    pos: &'a Position,
    current_dimension: usize,
    end_dimension: usize,
    current_iter: Option<DimensionIter<'a>>,
    range: std::ops::Range<i8>,
}

impl PositionIter<'_> {
    pub fn new(
        pos: &Position,
        start_dimension: usize,
        end_dimension: usize,
        range: std::ops::Range<i8>,
    ) -> PositionIter {
        PositionIter {
            pos,
            current_dimension: start_dimension,
            end_dimension,
            current_iter: DimensionIter::new(pos, start_dimension, range.clone()),
            range,
        }
    }
}

impl Iterator for PositionIter<'_> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.current_iter.as_mut()?.next() {
            return Some(next);
        }
        self.current_dimension += 1;
        if self.current_dimension > self.end_dimension {
            return None;
        }
        self.current_iter =
            DimensionIter::new(self.pos, self.current_dimension, self.range.clone());
        self.next()
    }
}

fn spawn_board(mut commands: Commands, dimensions: Res<Dimensions>) {
    commands.spawn((
        Name::new("Board Root"),
        Transform::default(),
        DimensionSpawner::new(**dimensions),
    ));
}

#[derive(Resource, Deref, DerefMut, Reflect)]
pub struct Dimensions(usize);

#[derive(Resource)]
pub struct BoardState {
    captured: Option<Entity>,
    board: bevy::platform::collections::HashMap<Position, Entity>,
}

impl FromWorld for BoardState {
    fn from_world(_world: &mut World) -> Self {
        Self::new()
    }
}

impl BoardState {
    pub fn new() -> Self {
        Self {
            captured: None,
            board: bevy::platform::collections::HashMap::default(),
        }
    }

    pub fn get(&self, position: &Position) -> Option<Entity> {
        self.board.get(position).copied()
    }

    pub fn set(&mut self, position: Position, team: Entity) {
        self.board.insert(position, team);
    }

    pub fn move_piece(&mut self, from: &Position, to: &Position) {
        if let Some(team) = self.board.remove(from) {
            self.captured = self.board.insert(to.clone(), team);
        }
    }
}

fn captured_piece(mut state: ResMut<BoardState>, mut commands: Commands) {
    if let Some(captured) = state.captured.take() {
        commands.entity(captured).despawn();
    }
}
