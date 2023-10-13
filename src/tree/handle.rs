use super::{error::Error, Node};
use embedded_io::blocking::{Read, Write};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use storage::Storage;

pub struct NodeReadHandle<'a, K, V, S> {
    pub(crate) id: u64,
    pub(crate) node: Node<K, V>,
    storage: &'a S,
}

impl<'a, K, V, S> Deref for NodeReadHandle<'a, K, V, S> {
    type Target = Node<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, K, V, S> NodeReadHandle<'a, K, V, S> {
    pub fn new(id: u64, node: Node<K, V>, storage: &S) -> Self {
        Self { id, node, storage }
    }

    pub fn open(id: u64, storage: &'a mut S) -> Result<Self, Error>
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

        Ok(Self {
            id,
            node: bincode::deserialize(&ser)?,
            storage,
        })
    }
}

pub struct NodeWriteHandle<'a, K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Storage<Id = u64>,
{
    id: u64,
    node: Node<K, V>,
    storage: &'a mut S,
}

impl<'a, K, V, S> NodeWriteHandle<'a, K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Storage<Id = u64>,
{
    pub fn create(node: Node<K, V>, storage: &'a mut S) -> Result<u64, Error> {
        let id = storage.alloc_id().map_err(|_| Error::Storage)?;
        let handle = Self { id, node, storage };
        Ok(id)
    }

    pub fn open(id: u64, storage: &'a mut S) -> Result<Self, Error>
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

        Ok(Self {
            id,
            node: bincode::deserialize(&ser)?,
            storage,
        })
    }

    pub fn close(&mut self) -> Result<(), Error>
    where
        K: Serialize,
        V: Serialize,
        S: Storage<Id = u64>,
    {
        let ser = bincode::serialize(&self.node)?;

        self.storage
            .write_handle(&self.id)
            .map_err(|_| Error::Storage)?
            .write_all(&ser)
            .map_err(|_| Error::Storage)?;

        Ok(())
    }
}

impl<'a, K, V, S> Deref for NodeWriteHandle<'a, K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Storage<Id = u64>,
{
    type Target = Node<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, K, V, S> DerefMut for NodeWriteHandle<'a, K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Storage<Id = u64>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<'a, K, V, S> Drop for NodeWriteHandle<'a, K, V, S>
where
    K: Serialize,
    V: Serialize,
    S: Storage<Id = u64>,
{
    fn drop(&mut self) {
        self.close().unwrap();
    }
}
