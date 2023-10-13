pub mod error;
pub mod handle;
pub mod node;

use error::Error;
use handle::{NodeReadHandle, NodeWriteHandle};
use node::Node;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};
use storage::Storage;

const DEFAULT_DEGREE: usize = 2;

pub struct BTree<K, V, S> {
    len: usize,
    degree: usize,
    root: u64,
    storage: S,
    pd: PhantomData<(K, V)>,
}

impl<K, V, S> BTree<K, V, S> {
    pub fn new(storage: S) -> Result<Self, Error>
    where
        K: Serialize,
        V: Serialize,
        S: Storage<Id = u64>,
    {
        Self::with_degree(storage, DEFAULT_DEGREE)
    }

    pub fn with_degree(mut storage: S, degree: usize) -> Result<Self, Error>
    where
        K: Serialize,
        V: Serialize,
        S: Storage<Id = u64>,
    {
        Ok(Self {
            len: 0,
            degree,
            root: NodeWriteHandle::create(Node::<K, V>::new(), &mut storage)?,
            storage,
            pd: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, k: &K) -> Result<bool, Error>
    where
        for<'de> K: Ord + Deserialize<'de>,
        for<'de> V: Deserialize<'de>,
    {
        self.get(k).map(|res| res.is_some())
    }

    pub fn get(&self, k: &K) -> Result<Option<&V>, Error>
    where
        for<'de> K: Ord + Deserialize<'de>,
        for<'de> V: Deserialize<'de>,
    {
        NodeReadHandle::open(self.root, self.storage)?
            .get(k)
            .map(|(idx, node)| &node.vals[idx])
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.root.get_mut(k).map(|(idx, node)| &mut node.vals[idx])
    }

    pub fn get_key_value(&self, k: &K) -> Option<(&K, &V)> {
        self.root
            .get(k)
            .map(|(idx, node)| (&node.keys[idx], &node.vals[idx]))
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.root.is_full(self.degree) {
            let mut new_root = Node::new();
            std::mem::swap(&mut self.root, &mut new_root);
            self.root.children.push(new_root);
            self.root.split_child(0, self.degree);
        }

        let res = self.root.insert_nonfull(k, v, self.degree);

        if res.is_none() {
            self.len += 1;
        }

        res
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.remove_entry(k).map(|(_, val)| val)
    }

    pub fn remove_entry(&mut self, k: &K) -> Option<(K, V)> {
        if let Some(entry) = self.root.remove(k, self.degree) {
            if !self.root.is_leaf() && self.root.is_empty() {
                self.root = self.root.children.pop().unwrap();
            }
            self.len -= 1;
            Some(entry)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.root = Node::new();
    }
}

impl<K, V, S> Debug for BTree<K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.root)
    }
}
