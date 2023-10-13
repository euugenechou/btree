use super::node::Node;

pub struct Iter<'a, K, V> {
    nodes: Vec<&'a Node<K, V>>,
    indices: Vec<usize>,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub(crate) fn new(mut root: &'a Node<K, V>) -> Self {
        let mut nodes = vec![];
        let mut indices = vec![];

        if !root.is_empty() {
            while !root.is_leaf() {
                nodes.push(root);
                indices.push(0);
                root = root.children.first().unwrap();
            }
            nodes.push(root);
            indices.push(0);
        }

        Self { nodes, indices }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes.is_empty() {
            return None;
        }

        let node = *self.nodes.last().unwrap();
        let mut idx = *self.indices.last().unwrap();

        let key = node.keys.get(idx).unwrap();
        let val = node.vals.get(idx).unwrap();

        idx += 1;
        *self.indices.last_mut().unwrap() = idx;

        if idx == node.len() {
            self.nodes.truncate(self.nodes.len() - 1);
            self.indices.truncate(self.indices.len() - 1);
        }

        if idx < node.children.len() {
            let mut n = &node.children[idx];

            while !n.is_leaf() {
                self.nodes.push(n);
                self.indices.push(0);
                n = n.children.first().unwrap();
            }

            self.nodes.push(n);
            self.indices.push(0);
        }

        Some((key, val))
    }
}

pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Keys<'a, K, V> {
    pub(crate) fn new(inner: Iter<'a, K, V>) -> Self {
        Self { inner }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, _)| k)
    }
}

pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Values<'a, K, V> {
    pub(crate) fn new(inner: Iter<'a, K, V>) -> Self {
        Self { inner }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}
