use bevy::prelude::*;
use parking_lot::RwLock;
use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};
use weak_table::WeakValueHashMap;

/// This is used to keep a reference to a mesh handle in each chunk entity. This ensures that the WeakMap
/// we use to look up mesh handles can drop handles that no chunks are using anymore.
#[derive(Component)]
pub(crate) struct MeshRef(pub Arc<Handle<Mesh>>);

type WeakMeshMap = WeakValueHashMap<u64, Weak<Handle<Mesh>>>;

/// MeshCache uses a weak map to keep track of mesh handles generated for a certain configuration of voxels.
/// Using this map, we can avoid generating the same mesh multiple times, and reusing mesh handles
/// should allow Bevy to automatically batch draw identical chunks (large flat areas for example)
#[derive(Resource, Clone)]
pub(crate) struct MeshCache<C> {
    map: Arc<RwLock<WeakMeshMap>>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Send + Sync + 'static> MeshCache<C> {
    pub fn apply_buffers(&self, insert_buffer: &mut MeshCacheInsertBuffer<C>) {
        if insert_buffer.len() == 0 {
            return;
        }

        if let Some(mut map) = self.map.try_write() {
            for (voxels, mesh) in insert_buffer.drain(..) {
                map.insert(voxels, mesh);
            }
            map.remove_expired();
        }
    }

    pub fn get(&self, voxels_hash: &u64) -> Option<Arc<Handle<Mesh>>> {
        self.map.read().get(voxels_hash)
    }

    pub fn get_map(&self) -> Arc<RwLock<WeakMeshMap>> {
        self.map.clone()
    }
}

impl<C> Default for MeshCache<C> {
    fn default() -> Self {
        Self {
            map: Arc::new(RwLock::new(WeakMeshMap::with_capacity(2000))),
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub(crate) struct MeshCacheInsertBuffer<C>(#[deref] Vec<(u64, Arc<Handle<Mesh>>)>, PhantomData<C>);
