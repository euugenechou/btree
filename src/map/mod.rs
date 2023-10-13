mod node;
#[cfg(test)]
mod tests;

use node::Node;
use std::{
    fmt::{self, Debug, Formatter},
    mem,
};

const DEFAULT_DEGREE: usize = 2;

pub struct BTreeMap<K, V> {
    len: usize,
    degree: usize,
    root: Node<K, V>,
}

impl<K, V> BTreeMap<K, V> {
    pub fn new() -> Self {
        Self::with_degree(DEFAULT_DEGREE)
    }

    pub fn with_degree(degree: usize) -> Self {
        Self {
            len: 0,
            degree,
            root: Node::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, k: &K) -> bool
    where
        K: Ord,
    {
        self.get(k).is_some()
    }

    pub fn get(&self, k: &K) -> Option<&V>
    where
        K: Ord,
    {
        self.root.get(k).map(|(idx, node)| &node.vals[idx])
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V>
    where
        K: Ord,
    {
        self.root.get_mut(k).map(|(idx, node)| &mut node.vals[idx])
    }

    pub fn get_key_value(&self, k: &K) -> Option<(&K, &V)>
    where
        K: Ord,
    {
        self.root
            .get(k)
            .map(|(idx, node)| (&node.keys[idx], &node.vals[idx]))
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Ord,
    {
        if self.root.is_full(self.degree) {
            let mut new_root = Node::new();
            mem::swap(&mut self.root, &mut new_root);
            self.root.children.push(new_root);
            self.root.split_child(0, self.degree);
        }

        let res = self.root.insert_nonfull(k, v, self.degree);

        if res.is_none() {
            self.len += 1;
        }

        res
    }

    pub fn remove(&mut self, k: &K) -> Option<V>
    where
        K: Ord,
    {
        self.remove_entry(k).map(|(_, val)| val)
    }

    pub fn remove_entry(&mut self, k: &K) -> Option<(K, V)>
    where
        K: Ord,
    {
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

impl<K, V> Debug for BTreeMap<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.root)
    }
}
