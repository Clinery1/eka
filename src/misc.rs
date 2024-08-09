use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;
use misc_utils::{
    KeyedVec,
    Key,
};
use std::{
    ops::{
        Index,
        IndexMut,
    },
    hash::Hash,
};


#[derive(Debug)]
pub struct IndexedItemStore<K: Hash + PartialEq + Eq + Key, V> {
    all: KeyedVec<K, V>,
    roots: IndexSet<K, FxBuildHasher>,
}
impl<K: Hash + PartialEq + Eq + Key, V> IndexedItemStore<K, V> {
    pub fn insert(&mut self, val: V)->K {
        self.all.insert(val)
    }

    pub fn add_root(&mut self, id: K) {
        self.roots.insert(id);
    }

    pub fn remove_root(&mut self, id: K) {
        self.roots.shift_remove(&id);
    }

    /// Iterates through the roots in order
    pub fn iter_roots(&self)->impl Iterator<Item = &K> {
        self.roots.iter()
    }
}
impl<K: Hash + PartialEq + Eq + Key, V> Index<K> for IndexedItemStore<K, V> {
    type Output = V;

    fn index(&self, id: K)->&V {
        &self.all[id]
    }
}
impl<K: Hash + PartialEq + Eq + Key, V> IndexMut<K> for IndexedItemStore<K, V> {
    fn index_mut(&mut self, id: K)->&mut V {
        &mut self.all[id]
    }
}
