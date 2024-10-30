use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, utils::HashMap};
use bevy_voxel_world::prelude::*;
use noise::{HybridMulti, NoiseFn, Perlin};

#[derive(Resource, Clone, Default)]
struct MainWorld;

impl VoxelWorldConfig for MainWorld {
    type Index = u8;

    fn spawning_distance(&self) -> u32 {
        25
    }

    fn voxel_lookup_delegate(&self) -> VoxelLookupDelegate<Self::Index> {
        Box::new(move |_chunk_pos| get_voxel_fn())
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelWorldPlugin::with_config(MainWorld))
        .add_systems(Startup, setup)
        .add_systems(Update, move_camera)
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-200.0, 180.0, -200.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // This tells bevy_voxel_world to use this cameras transform to calculate spawning area
        VoxelWorldCamera::<MainWorld>::default(),
    ));

    // Sun
    let cascade_shadow_config = CascadeShadowConfigBuilder { ..default() }.build();
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.1, 0.15), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });

    // Ambient light, same color as sun
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.98, 0.95, 0.82),
        brightness: 100.0,
    });
}

fn get_voxel_fn() -> Box<dyn FnMut(IVec3) -> WorldVoxel<u8> + Send + Sync> {
    // Set up some noise to use as the terrain height map
    let mut noise = HybridMulti::<Perlin>::new(1234);
    noise.octaves = 5;
    noise.frequency = 1.1;
    noise.lacunarity = 2.8;
    noise.persistence = 0.4;

    // We use this to cache the noise value for each y column so we only need
    // to calculate it once per x/z coordinate
    let mut cache = HashMap::<(i32, i32), f64>::new();

    // Then we return this boxed closure that captures the noise and the cache
    // This will get sent off to a separate thread for meshing by bevy_voxel_world
    Box::new(move |pos: IVec3| {
        // Sea level
        if pos.y < 1 {
            return WorldVoxel::Solid(3);
        }

        let [x, y, z] = pos.as_dvec3().to_array();

        // If y is less than the noise sample, we will set the voxel to solid
        let is_ground = y < match cache.get(&(pos.x, pos.z)) {
            Some(sample) => *sample,
            None => {
                let sample = noise.get([x / 1000.0, z / 1000.0]) * 50.0;
                cache.insert((pos.x, pos.z), sample);
                sample
            }
        };

        if is_ground {
            // Solid voxel of material type 0
            WorldVoxel::Solid(0)
        } else {
            WorldVoxel::Air
        }
    })
}

fn move_camera(
    time: Res<Time>,
    mut cam_transform: Query<&mut Transform, With<VoxelWorldCamera<MainWorld>>>,
) {
    cam_transform.single_mut().translation.x += time.delta_seconds() * 30.0;
    cam_transform.single_mut().translation.z += time.delta_seconds() * 60.0;
}
