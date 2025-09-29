use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::board::Index;

#[derive(Component, Reflect)]
#[require(Visibility)]
#[component(on_add = Self::on_add)]
pub struct DimensionSpawner {
    dimension: usize,
}

impl DimensionSpawner {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl DimensionSpawner {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let dimension = world
            .get::<DimensionSpawner>(ctx.entity)
            .expect("Just added DimensionSpawner")
            .dimension;
        match dimension {
            0 => {
                let mut position = super::Position::new(&world, ctx.entity);
                let mut commands = world.commands();
                for i in 0..8 {
                    position.add_dimension(0, i);
                    commands.spawn((
                        Name::new(format!("Cell {i}")),
                        Index(0),
                        Transform::from_translation(Vec3::new(0., 0.0, i as f32)),
                    ));
                }
            }
            1 => {
                let board_resource = world.resource::<super::BoardResource>();
                let cube_handle = board_resource.cube_handle.clone();
                let black_material = board_resource.black_material.clone();
                let white_material = board_resource.white_material.clone();
                let position = super::Position::new(&world, ctx.entity);
                #[cfg(debug_assertions)]
                {
                    use crate::board::Dimensions;

                    let dimensions = world.resource::<Dimensions>();
                    debug_assert!(
                        **dimensions == position.len(),
                        "Position did not find all parent dimensions. Found {}, expected {}",
                        position.len(),
                        **dimensions
                    );
                }
                let mut commands = world.commands();
                let material = if position.sum().is_multiple_of(2) {
                    black_material.clone()
                } else {
                    white_material.clone()
                };
                commands.spawn((
                    Name::new("Dimension 1"),
                    Transform::default(),
                    DimensionSpawner::new(0),
                    MeshMaterial3d(material),
                    Mesh3d(cube_handle.clone()),
                    ChildOf(ctx.entity),
                ));
            }
            2..=7 => {
                Self::spawn_dimension(world.commands(), ctx.entity, dimension);
            }
            _ => {
                world.commands().spawn((
                    Name::new(format!("Unrenderable Dimension Placeholder: {dimension}")),
                    DimensionSpawner::new(dimension - 1),
                ));
                error!("I don't know how to render more then 5 dimensions symmetrically");
            }
        }
    }

    fn spawn_dimension(mut commands: Commands, parent: Entity, dimension: usize) {
        debug_assert!(
            (2..=7).contains(&dimension),
            "Can only render dimensions 2 to 5"
        );
        let mut root = commands.entity(parent);
        root.with_children(|p| {
            for (step, index) in DimensionStep::new(dimension) {
                p.spawn((
                    Name::new(format!("Dimension {dimension}: {index}")),
                    Index(index as i8),
                    Transform::from_translation(step),
                    DimensionSpawner::new(dimension - 1),
                ));
            }
        });
    }
}

struct DimensionStep {
    index: usize,
    step: Vec3,
}

impl DimensionStep {
    fn new(dimension: usize) -> Self {
        let step = render_dimension_step_size(dimension).unwrap_or(Vec3::ZERO);
        Self { step, index: 0 }
    }
}

impl Iterator for DimensionStep {
    type Item = (Vec3, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 8 {
            return None;
        }
        let current = self.step * self.index as f32;
        self.index += 1;
        Some((current, self.index - 1))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (8 - self.index, Some(8 - self.index))
    }
}

impl ExactSizeIterator for DimensionStep {
    fn len(&self) -> usize {
        8 - self.index
    }
}

pub fn render_dimension_step_size(dimension: usize) -> Option<Vec3> {
    match dimension {
        1 => Some(Vec3::NEG_Z),
        2 => Some(Vec3::X),
        3 => Some(Vec3::Y * 5.),
        4 => Some(Vec3::Z * 9.),
        5 => Some(Vec3::X * 9.),
        6 => Some(Vec3::Z * 81.),
        7 => Some(Vec3::X * 81.),
        _ => {
            error!("I don't know how to render more then 7 dimensions symmetrically");
            None
        }
    }
}
