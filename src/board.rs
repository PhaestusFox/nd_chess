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
            .insert_resource(Dimensions(5))
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
struct Index(i8);

#[derive(Deref, Clone, Component, Hash, PartialEq, Eq, Default, Debug)]
pub struct Position(pub Vec<i8>);

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
    fn add_dimension(&mut self, dimension: usize, index: i8) {
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

    pub fn inc_in_place(&mut self, dimension: usize) {
        if let Some(last) = self.0.get_mut(dimension) {
            *last += 1;
        }
    }

    pub fn dec(mut self, dimension: usize) -> Self {
        if let Some(last) = self.0.get_mut(dimension) {
            *last -= 1;
        }
        self
    }

    pub fn dec_in_place(&mut self, dimension: usize) {
        if let Some(last) = self.0.get_mut(dimension) {
            *last -= 1;
        }
    }

    pub fn last(&self) -> i8 {
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

    pub fn all_but(&self, dim: usize, val: i8) -> bool {
        for (i, &v) in self.0.iter().enumerate() {
            if i != dim && v != val {
                return false;
            }
        }
        true
    }

    pub fn all(&self, val: i8) -> bool {
        for &v in self.0.iter() {
            if v != val {
                return false;
            }
        }
        true
    }

    pub fn add(&mut self, other: &Position) {
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            *a += *b;
        }
    }
    pub fn dec_all(&self) -> Position {
        let mut out = self.clone();
        for v in out.0.iter_mut() {
            *v -= 1;
        }
        out
    }
    pub fn inc_all(&self) -> Position {
        let mut out = self.clone();
        for v in out.0.iter_mut() {
            *v += 1;
        }
        out
    }
    pub fn is_valid(&self) -> bool {
        for &i in self.0.iter() {
            if !(0..=7).contains(&i) {
                return false;
            }
        }
        true
    }

    pub fn if_is(&self, dim: usize, val: i8) -> bool {
        if let Some(v) = self.get(dim)
            && val == *v
        {
            true
        } else {
            false
        }
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

pub struct DimensionIter<const DISTANCE: i8> {
    current: Position,
    dimension: usize,
    up: bool,
}

impl<const DISTANCE: i8> DimensionIter<DISTANCE> {
    pub fn new(dimensions: usize, dimension: usize, inc: bool) -> DimensionIter<DISTANCE> {
        debug_assert!(dimension < dimensions);
        DimensionIter {
            current: Position(vec![0; dimensions]),
            dimension,
            up: inc,
        }
    }
}

impl<const DISTANCE: i8> Iterator for DimensionIter<DISTANCE> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current[self.dimension] > DISTANCE || self.current[self.dimension] < -DISTANCE {
            return None;
        }
        let out = self.current.clone();
        if self.up {
            self.current.inc_in_place(self.dimension);
        } else {
            self.current.dec_in_place(self.dimension);
        }
        Some(out)
    }
}

pub struct PositionIter<const DISTANCE: i8> {
    iter: DimensionIter<DISTANCE>,
    current_dimension: usize,
    end_dimension: usize,
    was_inc: bool,
}

impl<const DISTANCE: i8> PositionIter<DISTANCE> {
    pub fn new(dimensions: usize) -> PositionIter<DISTANCE> {
        PositionIter {
            iter: DimensionIter::new(dimensions, 0, true),
            current_dimension: 0,
            end_dimension: dimensions - 1,
            was_inc: true,
        }
    }
    pub fn start_at(dimensions: usize, dimension: usize) -> PositionIter<DISTANCE> {
        PositionIter {
            iter: DimensionIter::new(dimensions, dimension, true),
            current_dimension: dimension,
            end_dimension: dimensions - 1,
            was_inc: true,
        }
    }
}

impl<const DISTANCE: i8> Iterator for PositionIter<DISTANCE> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_dimension > self.end_dimension {
            return None;
        }
        if let Some(next) = self.iter.next() {
            return Some(next);
        }
        if self.was_inc {
            self.was_inc = false;
            self.iter = DimensionIter::new(self.end_dimension + 1, self.current_dimension, false);
            self.iter.next();
        } else if self.current_dimension != self.end_dimension {
            self.current_dimension += 1;
            self.was_inc = true;
            self.iter = DimensionIter::new(self.end_dimension + 1, self.current_dimension, true);
            self.iter.next();
        } else {
            return None;
        }
        self.next()
    }
}

pub struct NewPositionIter<const DISTANCE: u8> {
    current: Position,
}

impl<const DISTANCE: u8> NewPositionIter<DISTANCE> {
    pub fn new(dimensions: usize) -> Self {
        Self {
            current: Position(vec![0; dimensions]),
        }
    }
}

impl<const DISTANCE: u8> Iterator for NewPositionIter<DISTANCE> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.last() > DISTANCE as i8 {
            return None;
        }
        let out = self.current.clone();
        self.current.0[0] += 1;
        for i in 0..(self.current.0.len() - 1) {
            if self.current.0[i] > DISTANCE as i8 {
                self.current.0[i] = 0;
                self.current.0[i + 1] += 1;
            }
        }
        Some(out)
    }
}

pub struct OffsetIter<'a, T: Iterator<Item = Position>> {
    base: &'a Position,
    iter: T,
}

impl<'a, T: Iterator<Item = Position>> OffsetIter<'a, T> {
    pub fn new(base: &'a Position, iter: T) -> Self {
        Self { base, iter }
    }
}

impl<T: Iterator<Item = Position>> Iterator for OffsetIter<'_, T> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.iter.next().map(|mut p| {
            p.add(self.base);
            p
        })?;
        if out.is_valid() {
            Some(out)
        } else {
            self.next()
        }
    }
}

fn spawn_board(mut commands: Commands, dimensions: Res<Dimensions>) {
    commands.spawn((
        Name::new("Board Root"),
        Transform::default(),
        DimensionSpawner::new(**dimensions),
    ));
}

pub trait WithOffset {
    fn with_offset(self, offset: &Position) -> OffsetIter<Self>
    where
        Self: Sized + Iterator<Item = Position>,
    {
        OffsetIter::new(offset, self)
    }
}

impl<T: Iterator<Item = Position>> WithOffset for T {}

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
