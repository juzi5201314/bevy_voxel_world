use std::sync::Arc;

use bevy::prelude::*;
use bevy_voxel_world::prelude::*;

#[derive(Resource, Clone, Default)]
struct MyWorld;

impl VoxelWorldConfig for MyWorld {
    type Index = u8;

    fn texture_index_mapper(&self) -> TextureIndexMapper<Self::Index> {
        Arc::new(|vox_mat| match vox_mat {
            0 => [1, 1, 0, 2, 3, 1].into(),
            1 => [3, 1, 0, 2, 1, 1].into(),
            2 => [1, 1, 0, 2, 1, 3].into(),
            3 | _ => [1, 3, 0, 2, 1, 1].into(),
        })
    }

    fn voxel_texture(&self) -> Option<(String, u32)> {
        Some(("example_voxel_texture.png".into(), 4))
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelWorldPlugin::with_config(MyWorld))
        .insert_resource(Time::<Fixed>::from_seconds(0.5))
        .add_systems(Startup, setup)
        .add_systems(Update, move_camera)
        .add_systems(FixedUpdate, set_solid_voxel)
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // This tells bevy_voxel_world to use this cameras transform to calculate spawning area
        VoxelWorldCamera::<MyWorld>::default(),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.98, 0.95, 0.82),
        brightness: 1000.0,
    });
}

fn set_solid_voxel(mut voxel_world: VoxelWorld<MyWorld>, mut voxel_type: Local<u8>) {
    if *voxel_type > 3 {
        *voxel_type = 0;
    } else {
        *voxel_type += 1;
    }

    voxel_world.set_voxel(IVec3::ZERO, WorldVoxel::Solid(*voxel_type));
}

// Rotate the camera around the origin
fn move_camera(time: Res<Time>, mut query: Query<&mut Transform, With<VoxelWorldCamera<MyWorld>>>) {
    let mut transform = query.single_mut();
    let time_seconds = time.elapsed_seconds();
    transform.translation.x = 25.0 * (time_seconds * 0.3).sin();
    transform.translation.z = 25.0 * (time_seconds * 0.3).cos();
    transform.look_at(Vec3::ZERO, Vec3::Y);
}
