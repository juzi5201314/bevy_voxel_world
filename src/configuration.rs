use std::hash::Hash;
use std::sync::Arc;

use crate::voxel::WorldVoxel;
use bevy::prelude::*;

pub type VoxelLookupFn<I> = Box<dyn FnMut(IVec3) -> WorldVoxel<I> + Send + Sync>;
pub type VoxelLookupDelegate<I> = Box<dyn Fn(IVec3) -> VoxelLookupFn<I> + Send + Sync>;
pub type TextureIndexMapper<I> = Arc<dyn Fn(I) -> FaceTextureIndex + Send + Sync>;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FaceTextureIndex {
    pub top: u32,
    pub left: u32,
    pub right: u32,
    pub front: u32,
    pub back: u32,
    pub bottom: u32,
}

#[derive(Default, PartialEq, Eq)]
pub enum ChunkDespawnStrategy {
    /// Despawn chunks that are further than `spawning_distance` away from the camera
    /// or outside of the viewport.
    #[default]
    FarAwayOrOutOfView,

    /// Only despawn chunks that are further than `spawning_distance` away from the camera.
    FarAway,
}

#[derive(Default, PartialEq, Eq)]
pub enum ChunkSpawnStrategy {
    /// Spawn chunks that are within `spawning_distance` of the camera
    /// and also inside the viewport.
    #[default]
    CloseAndInView,

    /// Spawn chunks that are within `spawning_distance` of the camera, regardless of whether
    /// they are in the viewport or not. Will only have an effect if the despawn strategy is
    /// `FarAway`. If this strategy is used a flood fill will be used to find unspawned chunks
    /// and therefore it might make sense to lower the `spawning_rays` option.
    Close,
}

/// `bevy_voxel_world` configuation structs need to implement this trait
pub trait VoxelWorldConfig: Resource + Default + Clone {
    type Index: Copy + Hash + PartialEq + Eq + Default + Send + Sync;

    /// Distance in chunks to spawn chunks around the camera
    fn spawning_distance(&self) -> u32 {
        10
    }

    /// Strategy for despawning chunks
    fn chunk_despawn_strategy(&self) -> ChunkDespawnStrategy {
        ChunkDespawnStrategy::default()
    }

    /// Strategy for spawning chunks
    /// This is only used if the despawn strategy is `FarAway`
    fn chunk_spawn_strategy(&self) -> ChunkSpawnStrategy {
        ChunkSpawnStrategy::default()
    }

    /// Maximum number of chunks that can get queued for spawning in a given frame.
    /// In some scenarios, reducing this number can help with performance, due to less
    /// thread contention.
    fn max_spawn_per_frame(&self) -> usize {
        10000
    }

    /// Number of rays to cast when spawning chunks. Higher values will result in more
    /// chunks being spawned per frame, but will also increase cpu load, and can lead to
    /// thread contention.
    fn spawning_rays(&self) -> usize {
        100
    }

    /// How far outside of the viewports spawning rays should get cast. Higher values will
    /// will reduce the likelyhood of chunks popping in, but will also increase cpu load.
    fn spawning_ray_margin(&self) -> u32 {
        25
    }

    /// Debugging aids
    fn debug_draw_chunks(&self) -> bool {
        false
    }

    /// A function that maps voxel materials to texture coordinates.
    /// The input is the material index, and the output is a slice of three indexes into an array texture.
    /// The three values correspond to the top, sides and bottom of the voxel. For example,
    /// if the slice is `[1,2,2]`, the top will use texture index 1 and the sides and bottom will use texture
    /// index 2.
    fn texture_index_mapper(&self) -> TextureIndexMapper<Self::Index> {
        Arc::new(|_| 0.into())
    }

    /// A function that returns a function that returns true if a voxel exists at the given position
    /// The delegate will be called every time a new chunk needs to be computed. The delegate should
    /// return a function that can be called to check if a voxel exists at a given position. This function
    /// needs to be thread-safe, since chunk computation happens on a separate thread.
    fn voxel_lookup_delegate(&self) -> VoxelLookupDelegate<Self::Index> {
        Box::new(|_| Box::new(|_| WorldVoxel::Unset))
    }

    /// A tuple of the path to the texture and the number of indexes in the texture. `None` if no texture is used.
    fn voxel_texture(&self) -> Option<(String, u32)> {
        None
    }

    /// Custom material will not get initialized if this returns false. When this is false,
    /// `VoxelWorldMaterialHandle` needs to be manually added with a reference to the material handle.
    ///
    /// This can be used for example if you need to wait for a texture image to load before
    /// the material can be used.
    fn init_custom_materials(&self) -> bool {
        true
    }

    fn init_root(&self, mut _commands: Commands, _root: Entity) {}
}

/// [x+, x-, y+, y-, z+, z-]
impl From<[u32; 6]> for FaceTextureIndex {
    fn from(value: [u32; 6]) -> Self {
        FaceTextureIndex {
            right: value[0],
            left: value[1],
            top: value[2],
            bottom: value[3],
            front: value[4],
            back: value[5],
        }
    }
}

/// [top, side, bottom]
impl From<[u32; 3]> for FaceTextureIndex {
    fn from(value: [u32; 3]) -> Self {
        FaceTextureIndex {
            left: value[1],
            right: value[1],
            top: value[0],
            bottom: value[1],
            front: value[1],
            back: value[2],
        }
    }
}

impl From<u32> for FaceTextureIndex {
    fn from(value: u32) -> Self {
        FaceTextureIndex {
            left: value,
            right: value,
            top: value,
            bottom: value,
            front: value,
            back: value,
        }
    }
}

#[derive(Resource, Clone, Default)]
pub struct DefaultWorld;

impl DefaultWorld {}

impl VoxelWorldConfig for DefaultWorld {
    type Index = u8;

    fn texture_index_mapper(&self) -> TextureIndexMapper<Self::Index> {
        Arc::new(|mat| match mat {
            0 => 0.into(),
            1 => 1.into(),
            2 => 2.into(),
            3 => 3.into(),
            _ => 0.into(),
        })
    }
}
