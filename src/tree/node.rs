use super::error::Error;
use embedded_io::blocking::{Read, Write};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    mem,
};
use storage::Storage;

#[derive(Serialize, Deserialize)]
pub struct Node<K, V> {
    pub(crate) id: u64,
    pub(crate) keys: Vec<K>,
    pub(crate) vals: Vec<V>,
    pub(crate) children: Vec<u64>,
}

impl<K, V> Node<K, V> {
    pub fn new<S>(storage: &mut S) -> Result<Self, Error>
    where
        K: Serialize,
        V: Serialize,
        S: Storage<Id = u64>,
    {
        let node = Self {
            id: storage.alloc_id().map_err(|_| Error::Storage)?,
            keys: Vec::new(),
            vals: Vec::new(),
            children: Vec::new(),
        };

        node.write(storage)?;

        Ok(node)
    }

    pub fn read<S>(id: u64, storage: &mut S) -> Result<Self, Error>
    where
        for<'de> K: Deserialize<'de>,
        for<'de> V: Deserialize<'de>,
        S: Storage<Id = u64>,
    {
        let mut ser = vec![];

        storage
            .read_handle(&id)
            .map_err(|_| Error::Storage)?
            .read_to_end(&mut ser)
            .map_err(|_| Error::Storage)?;

        Ok(bincode::deserialize(&ser)?)
    }

    pub fn write<S>(&self, storage: &mut S) -> Result<(), Error>
    where
        K: Serialize,
        V: Serialize,
        S: Storage<Id = u64>,
    {
        let ser = bincode::serialize(self)?;

        storage
            .write_handle(&self.id)
            .map_err(|_| Error::Storage)?
            .write_all(&ser)
            .map_err(|_| Error::Storage)?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn is_full(&self, degree: usize) -> bool {
        self.keys.len() == 2 * degree - 1
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn find_index(&self, k: &K) -> usize
    where
        K: Ord,
    {
        let mut size = self.len();
        let mut left = 0;
        let mut right = size;

        while left < right {
            let mid = left + size / 2;

            match self.keys[mid].cmp(k) {
                Ordering::Equal => return mid,
                Ordering::Less => left = mid + 1,
                Ordering::Greater => right = mid,
            }

            size = right - left;
        }

        left
    }

    pub fn get<S>(&self, k: &K, storage: &mut S) -> Result<Option<(usize, &Node<K, V>)>, Error>
    where
        for<'de> K: Ord + Deserialize<'de>,
        for<'de> V: Deserialize<'de>,
        S: Storage<Id = u64>,
    {
        let mut node = self;
        loop {
            let idx = node.find_index(k);
            if idx < node.len() && node.keys[idx] == *k {
                return Ok(Some((idx, node)));
            } else if node.is_leaf() {
                return Ok(None);
            } else {
                node = &Node::read(node.children[idx], storage)?;
            }
        }
    }

    pub fn get_mut<S>(
        &mut self,
        k: &K,
        storage: &mut S,
    ) -> Result<Option<(usize, &mut Node<K, V>)>, Error>
    where
        for<'de> K: Ord + Deserialize<'de>,
        for<'de> V: Deserialize<'de>,
        S: Storage<Id = u64>,
    {
        let mut node = self;
        loop {
            let idx = node.find_index(k);
            if idx < node.len() && node.keys[idx] == *k {
                return Ok(Some((idx, node)));
            } else if node.is_leaf() {
                return Ok(None);
            } else {
                node = &mut Node::read(node.children[idx], storage)?;
            }
        }
    }

    pub fn split_child<S>(
        &mut self,
        idx: usize,
        degree: usize,
        storage: &mut S,
    ) -> Result<(), Error>
    where
        for<'de> K: Ord + Serialize + Deserialize<'de>,
        for<'de> V: Serialize + Deserialize<'de>,
        S: Storage<Id = u64>,
    {
        let left = &mut Node::read(self.children[idx], storage)?;
        let mut right = Self::new(storage)?;

        // Move the largest keys and values from the left to the right.
        right.vals.extend(left.vals.drain(degree..));
        right.keys.extend(left.keys.drain(degree..));

        // Take the median (separator) key and value from the left.
        let key = left.keys.pop().expect("couldn't pop median key");
        let val = left.vals.pop().expect("couldn't pop median value");

        // Take the left's largest children as well if not a leaf.
        if !left.is_leaf() {
            right.children.extend(left.children.drain(degree..));
        }

        // Insert new key, value, and right child into the root.
        self.keys.insert(idx, key);
        self.vals.insert(idx, val);
        self.children.insert(idx + 1, right.id);

        // Persist changes.
        self.write(storage)?;
        left.write(storage)?;
        right.write(storage)?;

        Ok(())
    }

    pub fn insert_nonfull<S>(
        &mut self,
        k: K,
        mut v: V,
        degree: usize,
        storage: &mut S,
    ) -> Result<Option<V>, Error>
    where
        for<'de> K: Ord + Serialize + Deserialize<'de>,
        for<'de> V: Serialize + Deserialize<'de>,
        S: Storage<Id = u64>,
    {
        assert!(!self.is_full(degree));

        let mut node = self;
        loop {
            // Find index to insert key into or of the child to recurse down.
            let mut idx = node.find_index(&k);

            if node.is_leaf() {
                // Insert key and value into non-full node.
                if idx < node.len() && k == node.keys[idx] {
                    // The key already exists, so swap in the value.
                    mem::swap(&mut node.vals[idx], &mut v);
                    node.write(storage)?;
                    return Ok(Some(v));
                } else {
                    // The key doesn't exist yet.
                    node.keys.insert(idx, k);
                    node.vals.insert(idx, v);
                    node.write(storage)?;
                    return Ok(None);
                }
            } else {
                if node.children[idx].is_full(degree) {
                    // Split the child and determine which child to recurse down.
                    node.split_child(idx, degree, storage);
                    if node.keys[idx] < k {
                        idx += 1;
                    }
                }
                node = &mut Node::read(node.children[idx], storage)?;
            }
        }
    }

    fn min_key(&self) -> &K {
        let mut node = self;
        while !node.is_leaf() && !node.children.first().unwrap().is_empty() {
            node = node.children.first().unwrap();
        }
        node.keys.first().unwrap()
    }

    fn max_key(&self) -> &K {
        let mut node = self;
        while !node.is_leaf() && !node.children.last().unwrap().is_empty() {
            node = node.children.last().unwrap()
        }
        node.keys.last().unwrap()
    }

    pub fn remove<S>(
        &mut self,
        k: &K,
        degree: usize,
        storage: &mut S,
    ) -> Result<Option<(K, V)>, Error>
    where
        for<'de> K: Serialize + Ord + Deserialize<'de>,
        for<'de> V: Serialize + Deserialize<'de>,
        S: Storage<Id = u64>,
    {
        let mut idx = self.find_index(k);

        // Case 1: Key found in node and node is a leaf.
        if idx < self.len() && self.keys[idx] == *k && self.is_leaf() {
            let key = self.keys.remove(idx);
            let val = self.vals.remove(idx);
            self.write(storage)?;
            return Ok(Some((key, val)));
        }

        // Case 2: Key found in node and node is an internal node.
        if idx < self.len() && self.keys[idx] == *k && !self.is_leaf() {
            if self.children[idx].len() >= degree {
                // Case 2a: Child node that precedes k has at least t keys.
                let mut pred = Node::read(self.children[idx], storage)?;

                // Replace key with the predecessor key and recursively delete it.
                // Safety: we won't ever use the reference past this point.
                let pred_key = pred.max_key() as *const _;
                let (mut pred_key, mut pred_val) =
                    pred.remove(unsafe { &*pred_key }, degree, storage).unwrap();

                // The actual replacement.
                mem::swap(&mut self.keys[idx], &mut pred_key);
                mem::swap(&mut self.vals[idx], &mut pred_val);

                // Persist state.
                self.write(storage)?;
                pred.write(storage)?;

                return Ok(Some((pred_key, pred_val)));
            } else if self.children[idx + 1].len() >= degree {
                // Case 2b: Child node that succeeds k has at least t keys.
                let mut succ = Node::read(self.children[idx + 1], storage)?;

                // Replace key with the successor key and recursively delete it.
                // Safety: we don't ever use the reference past this point.
                let succ_key = succ.min_key() as *const _;
                let (mut succ_key, mut succ_val) = succ
                    .remove(unsafe { &*succ_key }, degree, storage)?
                    .unwrap();

                // The actual replacement.
                mem::swap(&mut self.keys[idx], &mut succ_key);
                mem::swap(&mut self.vals[idx], &mut succ_val);

                // Persist state.
                self.write(storage)?;
                succ.write(storage)?;

                return Ok(Some((succ_key, succ_val)));
            } else {
                // Case 2c: Successor and predecessor only have t - 1 keys.
                let key = self.keys.remove(idx);
                let val = self.vals.remove(idx);

                let mut succ = Node::read(self.children.remove(idx + 1), storage)?;
                let mut pred = Node::read(self.children[idx], storage)?;

                // Merge keys, values, and children into predecessor.
                pred.keys.push(key);
                pred.vals.push(val);
                pred.keys.append(&mut succ.keys);
                pred.vals.append(&mut succ.vals);
                pred.children.append(&mut succ.children);
                assert!(pred.is_full(degree));

                // Persist state.
                self.write(storage)?;
                pred.write(storage)?;
                succ.write(storage)?;

                return pred.remove(k, degree, storage);
            }
        }

        // If on a leaf, then no appropriate subtree contains the key.
        if self.is_leaf() {
            return Ok(None);
        }

        // Case 3: Key not found in internal node.
        if self.children[idx].len() + 1 == degree {
            if idx > 0 && self.children[idx - 1].len() >= degree {
                // Case 3a: Immediate left sibling has at least t keys.

                let mut mid = Node::read(self.children[idx], storage)?;
                let mut left = Node::read(self.children[idx - 1], storage)?;

                // Move key and value from parent down to child.
                let parent_key = self.keys.remove(idx - 1);
                let parent_val = self.vals.remove(idx - 1);
                mid.keys.insert(0, parent_key);
                mid.vals.insert(0, parent_val);

                // Move rightmost key and value in left sibling to parent.
                let left_key = left.keys.pop().unwrap();
                let left_val = left.vals.pop().unwrap();
                self.keys.insert(idx - 1, left_key);
                self.vals.insert(idx - 1, left_val);

                // Move rightmost child in left sibling to child.
                if !left.is_leaf() {
                    let child = left.children.pop().unwrap();
                    mid.children.insert(0, child);
                }

                // Persist state.
                self.write(storage)?;
                left.write(storage)?;
                mid.write(storage)?;
            } else if idx + 1 < self.children.len() && self.children[idx + 1].len() >= degree {
                // Case 3a: Immediate right sibling has at least t keys.

                let mut mid = Node::read(self.children[idx], storage)?;
                let mut right = Node::read(self.children[idx + 1], storage)?;

                // Move key and value from parent down to child.
                let parent_key = self.keys.remove(idx);
                let parent_val = self.vals.remove(idx);
                mid.keys.push(parent_key);
                mid.vals.push(parent_val);

                // Move leftmost key and value in right sibling to parent.
                let right_key = right.keys.remove(0);
                let right_val = right.vals.remove(0);
                self.keys.insert(idx, right_key);
                self.vals.insert(idx, right_val);

                // Move leftmost child in right sibling to child.
                if !right.is_leaf() {
                    let child = right.children.remove(0);
                    mid.children.push(child);
                }

                // Persist state.
                self.write(storage)?;
                right.write(storage)?;
                mid.write(storage)?;
            } else if idx > 0 {
                // Case 3b: Merge into left sibling.

                let mut mid = Node::read(self.children[idx], storage)?;
                let mut left = Node::read(self.children[idx - 1], storage)?;

                // Move key and value from parent down to left sibling (merged node).
                let parent_key = self.keys.remove(idx - 1);
                let parent_val = self.vals.remove(idx - 1);

                let mut mid_keys = mid.keys.drain(..).collect();
                let mut mid_vals = mid.vals.drain(..).collect();
                let mut mid_children = mid.children.drain(..).collect();

                left.keys.push(parent_key);
                left.vals.push(parent_val);

                // Merge all keys, values, and children from child into left sibling.
                left.keys.append(&mut mid_keys);
                left.vals.append(&mut mid_vals);
                left.children.append(&mut mid_children);

                // Remove the merged child.
                self.children.remove(idx);

                // Persist state.
                self.write(storage)?;
                mid.write(storage)?;
                left.write(storage)?;

                // The only case where you fix the child to recurse down.
                idx -= 1;
            } else if idx + 1 < self.children.len() {
                // Case 3b: Merge into right sibling.

                let mut mid = Node::read(self.children[idx], storage)?;
                let mut right = Node::read(self.children[idx + 1], storage)?;

                let parent_key = self.keys.remove(idx);
                let parent_val = self.vals.remove(idx);

                let mut right_keys = right.keys.drain(..).collect();
                let mut right_vals = right.vals.drain(..).collect();
                let mut right_children = right.children.drain(..).collect();

                mid.keys.push(parent_key);
                mid.vals.push(parent_val);
                mid.keys.append(&mut right_keys);
                mid.vals.append(&mut right_vals);
                mid.children.append(&mut right_children);

                // Remove the right sibling.
                self.children.remove(idx + 1);

                // Persist state.
                self.write(storage)?;
                right.write(storage)?;
                mid.write(storage)?;
            }
        }

        self.children[idx].remove(k, degree)
    }
}

impl<K, V> Debug for Node<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn fmt_tree<K, V>(
            f: &mut Formatter,
            node: &Node<K, V>,
            prefix: String,
            last: bool,
            root: bool,
        ) -> fmt::Result
        where
            K: Debug,
            V: Debug,
        {
            if !root {
                write!(
                    f,
                    "{}{}",
                    prefix,
                    if last {
                        "└─── "
                    } else {
                        "├─── "
                    }
                )?;
            }

            writeln!(f, "{:?}", node.keys)?;
            // writeln!(
            //     f,
            //     "{:?}",
            //     node.keys.iter().zip(node.vals.iter()).collect::<Vec<_>>()
            // )?;

            if !node.is_leaf() {
                for (i, c) in node.children.iter().enumerate() {
                    let next_prefix = if root {
                        format!("{prefix}")
                    } else if last {
                        format!("{prefix}     ")
                    } else {
                        format!("{prefix}│    ")
                    };

                    fmt_tree(f, c, next_prefix, i + 1 == node.children.len(), false)?;
                }
            }

            Ok(())
        }

        fmt_tree(f, self, String::new(), true, true)
    }
}
