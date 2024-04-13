use std::{collections::hash_map, fmt::Display, hash::Hash};

use kairos_trie::{KeyHash, PortableHash};

use crate::errors::Never;

pub trait Entry {
    type Key;
    type Value;
    type Error: Display + Send + Clone + 'static;
    type EntryApi<'a>: EntryApi<'a, Self::Key, Self::Value>
    where
        Self: 'a;

    fn entry(&mut self, key: Self::Key) -> Result<Self::EntryApi<'_>, Self::Error>;

    fn get(&self, key: &Self::Key) -> Result<Option<&Self::Value>, Self::Error>;
}

pub trait EntryApi<'a, K, V>: Sized {
    fn key(&self) -> &K;
    fn get(&self) -> Option<&V>;
    fn get_mut(&mut self) -> Option<&mut V>;
    fn insert(self, value: V) -> &'a mut V;
    fn or_insert_with_key(self, default: impl FnOnce(&K) -> V) -> &'a mut V;
    fn or_insert_with(self, default: impl FnOnce() -> V) -> &'a mut V {
        self.or_insert_with_key(|_| default())
    }
    fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with_key(|_| default)
    }
    fn and_modify(mut self, f: impl FnOnce(&mut V)) -> Self {
        self.get_mut().map(f);
        self
    }
    fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(|| Default::default())
    }
}

impl<K: Eq + Hash, V> Entry for hash_map::HashMap<K, V> {
    type Key = K;
    type Value = V;
    type Error = Never;
    type EntryApi<'a> = hash_map::Entry<'a, K, V> where K:'a, V: 'a;

    fn entry(&mut self, key: K) -> Result<Self::EntryApi<'_>, Self::Error> {
        Ok(self.entry(key))
    }

    fn get(&self, key: &K) -> Result<Option<&Self::Value>, Self::Error> {
        Ok(self.get(key))
    }
}

impl<'a, K: 'a, V: 'a> EntryApi<'a, K, V> for hash_map::Entry<'a, K, V> {
    fn key(&self) -> &K {
        self.key()
    }

    fn get(&self) -> Option<&V> {
        match self {
            hash_map::Entry::Occupied(e) => Some(e.get()),
            hash_map::Entry::Vacant(_) => None,
        }
    }

    fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            hash_map::Entry::Occupied(e) => Some(e.get_mut()),
            hash_map::Entry::Vacant(_) => None,
        }
    }

    fn insert(self, value: V) -> &'a mut V {
        match self {
            hash_map::Entry::Occupied(e) => {
                let v = e.into_mut();
                *v = value;
                v
            }
            hash_map::Entry::Vacant(e) => e.insert(value),
        }
    }

    fn or_insert_with_key(self, default: impl FnOnce(&K) -> V) -> &'a mut V {
        self.or_insert_with_key(default)
    }

    fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        self.and_modify(f)
    }
}

impl<S: kairos_trie::stored::Store<V>, V: PortableHash + Clone> Entry
    for kairos_trie::Transaction<S, V>
{
    type Key = KeyHash;
    type Value = V;
    type Error = kairos_trie::TrieError;
    type EntryApi<'a> = kairos_trie::Entry<'a, V> where V: 'a, S: 'a;

    fn entry(&mut self, key: Self::Key) -> Result<Self::EntryApi<'_>, Self::Error> {
        self.entry(&key)
    }

    fn get(&self, key: &Self::Key) -> Result<Option<&Self::Value>, Self::Error> {
        self.get(key)
    }
}

impl<'a, V: 'a> EntryApi<'a, KeyHash, V> for kairos_trie::Entry<'a, V> {
    fn key(&self) -> &KeyHash {
        self.key()
    }

    fn get(&self) -> Option<&V> {
        self.get()
    }

    fn get_mut(&mut self) -> Option<&mut V> {
        self.get_mut()
    }

    fn insert(self, value: V) -> &'a mut V {
        self.insert(value)
    }

    fn or_insert_with_key(self, default: impl FnOnce(&KeyHash) -> V) -> &'a mut V {
        self.or_insert_with_key(default)
    }

    fn or_insert_with(self, default: impl FnOnce() -> V) -> &'a mut V {
        self.or_insert_with(default)
    }

    fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert(default)
    }

    fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        self.and_modify(f)
    }

    fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_default()
    }
}
