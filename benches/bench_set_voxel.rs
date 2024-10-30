use bevy::prelude::*;
use bevy_voxel_world::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn _setup_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, VoxelWorldPlugin::<DefaultWorld>::minimal()));
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            VoxelWorldCamera::<DefaultWorld>::default(),
        ));
    });

    app
}

#[derive(Resource)]
struct Positions(Vec<IVec3>);

fn criterion_benchmark(c: &mut Criterion) {
    let positions = (0..10_000).map(|i| IVec3::new(i, i, i)).collect::<Vec<_>>();
    let mut app = _setup_app();
    app.insert_resource(Positions(positions));
    app.add_systems(
        Update,
        |mut voxel_world: VoxelWorld<DefaultWorld>, positions: Res<Positions>| {
            let test_voxel = WorldVoxel::Solid(1);

            for pos in &positions.0 {
                voxel_world.set_voxel(black_box(*pos), test_voxel);
            }
        },
    );
    app.update();
    c.bench_function("set 10k voxels pre update", |b| b.iter(|| app.update()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
