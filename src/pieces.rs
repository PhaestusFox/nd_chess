use bevy::ecs::entity;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::{ecs::component::HookContext, render::mesh::VertexAttributeValues};

use bevy::prelude::*;

use crate::board::{self, BoardState, NewPositionIter, Position, PositionIter};
use crate::board::{DimensionIter, WithOffset};

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PieceAssets>()
            .init_resource::<PossibleMoveAssets>();
        app.add_systems(Startup, (spawn_pieces, spawn_select_indicator));
        app.add_systems(
            Update,
            (
                update_piece_position,
                display_selected_piece,
                clean_up_possible_moves,
            ),
        );
        app.insert_resource(Team::White);
        app.add_observer(only_select_one);
        app.add_observer(select_piece);
        app.add_observer(display_possible_moves)
            .add_observer(make_move);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
#[component(on_insert = Self::on_insert)]
#[require(Team, Position)]
pub enum ChessPiece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Resource, Default)]
pub enum Team {
    #[default]
    White,
    Black,
}

impl Team {
    pub fn opposite(&self) -> Self {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
        }
    }
}

impl ChessPiece {
    pub fn all_possible_moves(
        &self,
        position: &Position,
        team: Team,
        board: &BoardState,
        pieces: &Query<&Team>,
    ) -> Vec<Position> {
        let dimensions = position.len();
        let mut moves = Vec::new();
        match self {
            ChessPiece::Pawn => {
                if team == Team::White {
                    // try moving forward
                    let next = position.clone().inc(0);
                    if board.get(&next).is_none() {
                        moves.push(next.clone());
                        if position[0] == 1 {
                            let next = next.clone().inc(0);
                            if board.get(&next).is_none() {
                                moves.push(next);
                            }
                        }
                    }
                    // try capturing diagonally
                    for pos in PositionIter::<1>::start_at(dimensions, 1).with_offset(next.clone())
                    {
                        if pos == next {
                            continue;
                        }
                        if let Some(entity) = board.get(&pos)
                            && let Ok(Team::Black) = pieces.get(entity)
                        {
                            moves.push(pos);
                        }
                    }
                } else {
                    // try moving forward
                    let next = position.clone().dec(0);
                    if board.get(&next).is_none() {
                        moves.push(next.clone());
                        if position[0] == 6 {
                            let next = next.clone().dec(0);
                            if board.get(&next).is_none() {
                                moves.push(next);
                            }
                        }
                    }
                    // try capturing diagonally
                    for pos in PositionIter::<1>::start_at(dimensions, 1).with_offset(next.clone())
                    {
                        if pos == next {
                            continue;
                        }
                        if let Some(entity) = board.get(&pos)
                            && let Ok(Team::White) = pieces.get(entity)
                        {
                            moves.push(pos);
                        }
                    }
                }
            }
            ChessPiece::King => {
                for pos in NewPositionIter::<2>::new(position.len()).with_offset(position.dec_all())
                {
                    if pos == *position {
                        continue;
                    }
                    if let Some(other) = board.get(&pos)
                        && let Ok(other_team) = pieces.get(other)
                        && *other_team == team
                    {
                        continue;
                    }
                    moves.push(pos);
                }
            }
            ChessPiece::Rook => {
                for axis in 0..dimensions {
                    for next in DimensionIter::<7>::new(position.len(), axis, true)
                        .with_offset(position.clone())
                    {
                        if next == *position {
                            continue;
                        }
                        if let Some(other) = board.get(&next)
                            && let Ok(other_team) = pieces.get(other)
                        {
                            if *other_team != team {
                                moves.push(next.clone());
                            }
                            break;
                        }
                        moves.push(next.clone());
                    }
                    for next in DimensionIter::<7>::new(position.len(), axis, false)
                        .with_offset(position.clone())
                    {
                        if next == *position {
                            continue;
                        }
                        if let Some(other) = board.get(&next)
                            && let Ok(other_team) = pieces.get(other)
                        {
                            if *other_team != team {
                                moves.push(next.clone());
                            }
                            break;
                        }
                        moves.push(next.clone());
                    }
                }
            }
            ChessPiece::Bishop => {}
            _ => {
                error!("{:?}: Not implemented yet", self);
            }
        }
        moves
    }

    pub fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let piece = *world
            .get::<ChessPiece>(ctx.entity)
            .expect("Just added ChessPiece");
        let position = world
            .get::<Position>(ctx.entity)
            .expect("Just added ChessPiece, must have Position")
            .clone();
        let assets = world.resource::<PieceAssets>();
        let mesh = match piece {
            ChessPiece::Pawn => assets.meshes[0].clone(),
            ChessPiece::Rook => assets.meshes[1].clone(),
            ChessPiece::King => assets.meshes[2].clone(),
            ChessPiece::Bishop => assets.meshes[3].clone(),
            ChessPiece::Knight => assets.meshes[4].clone(),
            ChessPiece::Queen => assets.meshes[5].clone(),
        };
        let mut commands = world.commands();
        commands
            .entity(ctx.entity)
            .insert((Name::new(format!("{piece:?}")), Mesh3d(mesh)));
        world.resource_mut::<BoardState>().set(position, ctx.entity);
    }
}

#[derive(Resource)]
struct PieceAssets {
    meshes: Vec<Handle<Mesh>>,
    white_material: Handle<StandardMaterial>,
    black_material: Handle<StandardMaterial>,
}

impl FromWorld for PieceAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let cube = meshes.add(Mesh::from(Cuboid::from_length(0.8)));
        let sphere = meshes.add(Mesh::from(Sphere::new(0.8)));
        let cylinder = meshes.add(Mesh::from(Cylinder::new(0.4, 0.8)));
        let cone = meshes.add(Mesh::from(Cone::new(0.8, 0.8)));
        let torus = meshes.add(Mesh::from(bevy::math::primitives::Torus::new(0.4, 0.8)));
        let capsule = meshes.add(Mesh::from(bevy::math::primitives::Capsule3d::new(0.4, 0.4)));
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let white_material = materials.add(StandardMaterial::default());
        let black_material = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            ..Default::default()
        });
        Self {
            meshes: vec![cube, cylinder, sphere, cone, capsule, torus],
            white_material,
            black_material,
        }
    }
}

fn spawn_pieces(
    mut commands: Commands,
    dimensions: Res<super::board::Dimensions>,
    assets: Res<PieceAssets>,
) {
    for position in PieceIter::new(**dimensions) {
        if position[0] == 1 {
            let mut piece = commands.spawn((
                Name::new("White Pawn"),
                position.clone(),
                ChessPiece::Pawn,
                Team::White,
                MeshMaterial3d(assets.white_material.clone()),
            ));
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        } else if position[0] == 6 {
            let mut piece = commands.spawn((
                Name::new("Black Pawn"),
                position.clone(),
                ChessPiece::Pawn,
                Team::Black,
                MeshMaterial3d(assets.black_material.clone()),
            ));
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        }
        if position[0] != 0 && position[0] != 7 {
            continue;
        }
        let team = if position[0] == 0 {
            Team::White
        } else {
            Team::Black
        };
        // king check
        if position.all_but(0, 4) {
            let mut piece = commands.spawn((position.clone(), ChessPiece::King, team));
            if team == Team::White {
                piece.insert((
                    Name::new("White King"),
                    MeshMaterial3d(assets.white_material.clone()),
                ));
            } else {
                piece.insert((
                    Name::new("Black King"),
                    MeshMaterial3d(assets.black_material.clone()),
                ));
            }
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        }
        // rook check
        if position.if_is(1, 0) || position.if_is(1, 7) {
            let mut piece = commands.spawn((position.clone(), ChessPiece::Rook, team));
            if team == Team::White {
                piece.insert((
                    Name::new("White Rook"),
                    MeshMaterial3d(assets.white_material.clone()),
                ));
            } else {
                piece.insert((
                    Name::new("Black Rook"),
                    MeshMaterial3d(assets.black_material.clone()),
                ));
            }
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        }
        // bishop check
        if position.if_is(1, 2) || position.if_is(1, 5) {
            let mut piece = commands.spawn((position.clone(), ChessPiece::Bishop, team));
            if team == Team::White {
                piece.insert((
                    Name::new("White Bishop"),
                    MeshMaterial3d(assets.white_material.clone()),
                ));
            } else {
                piece.insert((
                    Name::new("Black Bishop"),
                    MeshMaterial3d(assets.black_material.clone()),
                ));
            }
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        }
        // knight check
        if position.if_is(1, 1) || position.if_is(1, 6) {
            let mut piece = commands.spawn((position.clone(), ChessPiece::Knight, team));
            if team == Team::White {
                piece.insert((
                    Name::new("White Knight"),
                    MeshMaterial3d(assets.white_material.clone()),
                ));
            } else {
                piece.insert((
                    Name::new("Black Knight"),
                    MeshMaterial3d(assets.black_material.clone()),
                ));
            }
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        }
        // Queen check
        if position.if_is(1, 3) {
            let mut piece = commands.spawn((position.clone(), ChessPiece::Queen, team));
            if team == Team::White {
                piece.insert((
                    Name::new("White Queen"),
                    MeshMaterial3d(assets.white_material.clone()),
                ));
            } else {
                piece.insert((
                    Name::new("Black Queen"),
                    MeshMaterial3d(assets.black_material.clone()),
                ));
            }
            if position.is_visible(**dimensions) {
                piece.insert(Visibility::Visible);
            } else {
                piece.insert(Visibility::Hidden);
            }
            continue;
        }
    }
}

struct PieceIter {
    current: super::board::Position,
}

impl PieceIter {
    fn new(dimensions: usize) -> Self {
        let current = super::board::Position(vec![0; dimensions]);
        Self { current }
    }
}

impl Iterator for PieceIter {
    type Item = super::board::Position;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current[0] > 7 {
            return None;
        }
        let out = self.current.clone();
        *self.current.0.last_mut()? += 1;
        'out: loop {
            for i in (0..self.current.0.len()).rev() {
                if self.current[i] > 7 && i != 0 {
                    self.current.0[i] = 0;
                    self.current.0[i - 1] += 1;
                    continue 'out;
                }
            }
            break;
        }
        Some(out)
    }
}

fn update_piece_position(
    mut query: Query<(&super::board::Position, &mut Transform), Changed<super::board::Position>>,
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation = position.to_translation();
    }
}

#[derive(Component)]
struct Selected;

fn only_select_one(
    trigger: Trigger<OnAdd, Selected>,
    selected: Populated<Entity, With<Selected>>,
    mut commands: Commands,
) {
    for entity in selected.iter() {
        if trigger.target() == entity {
            continue;
        }
        commands.entity(entity).remove::<Selected>();
    }
}

fn select_piece(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    can_select: Query<(), With<ChessPiece>>,
    selected: Query<Entity, With<Selected>>,
) {
    if can_select.get(trigger.target()).is_err() {
        return;
    }
    println!("Clicked on {:?}", trigger.target());
    if selected.contains(trigger.target()) {
        commands.entity(trigger.target()).remove::<Selected>();
    } else {
        commands.entity(trigger.target()).insert(Selected);
    }
}

#[derive(Component)]
struct SelectIndicator;

fn spawn_select_indicator(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        Name::new("Select Indicator"),
        Mesh3d(meshes.add(Mesh::from(Cone::new(0.25, 1.)))),
        Transform::from_translation(Vec3::Y)
            .with_rotation(Quat::from_rotation_x(180f32.to_radians())),
        MeshMaterial3d::<StandardMaterial>(Handle::default()),
        SelectIndicator,
        Visibility::Hidden,
    ));
}

fn display_selected_piece(
    has_selected: Query<(), With<Selected>>,
    selected: Query<Entity, Added<Selected>>,
    mut select_indicator: Single<(Entity, &mut Visibility), With<SelectIndicator>>,
    mut commands: Commands,
) {
    if has_selected.is_empty() {
        *select_indicator.1 = Visibility::Hidden;
    } else if let Some(entity) = selected.iter().last() {
        *select_indicator.1 = Visibility::Visible;
        commands.entity(select_indicator.0).insert(ChildOf(entity));
    }
}

#[derive(Component)]
struct PossibleMove;

#[derive(Resource)]
struct PossibleMoveAssets {
    possible_move_material: Handle<StandardMaterial>,
    possible_move_mesh: Handle<Mesh>,
}

impl FromWorld for PossibleMoveAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let possible_move_material = materials.add(StandardMaterial {
            base_color: Color::linear_rgba(0.0, 1.0, 0.0, 0.5),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mut mesh = Cylinder::new(0.2, 0.2)
            .mesh()
            .anchor(bevy::render::mesh::CylinderAnchor::Bottom)
            .build();
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).and_then(|p| {
            let VertexAttributeValues::Float32x3(positions) = p else {
                return None;
            };
            for pos in positions {
                pos[1] += 1.;
            }
            Some(())
        });
        let possible_move_mesh = meshes.add(mesh);
        Self {
            possible_move_material,
            possible_move_mesh,
        }
    }
}

fn clean_up_possible_moves(
    removed: Query<(), With<Selected>>,
    old_moves: Populated<Entity, With<PossibleMove>>,
    mut commands: Commands,
) {
    if removed.is_empty() {
        for old in old_moves.iter() {
            commands.entity(old).despawn();
        }
    }
}

fn display_possible_moves(
    selected: Trigger<OnAdd, Selected>,
    old_moves: Query<Entity, With<PossibleMove>>,
    pieces: Query<(&Position, &ChessPiece, &Team)>,
    mut commands: Commands,
    assets: Res<PossibleMoveAssets>,
    board: Res<BoardState>,
) {
    for old in &old_moves {
        commands.entity(old).despawn();
    }
    let (position, piece, team) = pieces
        .get(selected.target())
        .expect("Just added Selected, must have piece");
    for possible in piece.all_possible_moves(
        position,
        *team,
        &board,
        &pieces.clone().transmute_lens().query(),
    ) {
        commands.spawn((
            Name::new("Possible Move"),
            possible.clone(),
            PossibleMove,
            Mesh3d(assets.possible_move_mesh.clone()),
            MeshMaterial3d(assets.possible_move_material.clone()),
        ));
    }
}

fn make_move(
    trigger: Trigger<Pointer<Click>>,
    mut selected: Single<(Entity, &mut Position), With<Selected>>,
    can_move: Query<&Position, (With<PossibleMove>, Without<Selected>)>,
    mut commands: Commands,
    mut board: ResMut<BoardState>,
) {
    let Ok(move_to) = can_move.get(trigger.target()) else {
        return;
    };
    board.move_piece(&selected.1, move_to);
    *selected.1 = move_to.clone();
    commands.entity(selected.0).remove::<Selected>();
}
