//! A YAML mapping and its iterator types.

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use crate::yaml_value::YamlValue;

/// A YAML mapping in which the keys and values are both `YamlValue`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Mapping {
    map: IndexMap<YamlValue, YamlValue>,
}

impl Mapping {
    /// Creates an empty YAML map.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty YAML map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Mapping {
            map: IndexMap::with_capacity(capacity),
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// into the map. The map may reserve more space to avoid frequent
    /// allocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    /// Shrinks the capacity of the map as much as possible. It will drop down
    /// as much as possible while maintaining the internal rules and possibly
    /// leaving some space in accordance with the resize policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    /// Inserts a key-value pair into the map. If the key already existed, the
    /// old value is returned.
    #[inline]
    pub fn insert(&mut self, k: YamlValue, v: YamlValue) -> Option<YamlValue> {
        self.map.insert(k, v)
    }

    /// Checks if the map contains the given key.
    #[inline]
    pub fn contains_key(&self, k: &YamlValue) -> bool {
        self.map.contains_key(k)
    }

    /// Returns the value corresponding to the key in the map.
    #[inline]
    pub fn get(&self, k: &YamlValue) -> Option<&YamlValue> {
        self.map.get(k)
    }

    /// Returns the mutable reference corresponding to the key in the map.
    #[inline]
    pub fn get_mut(&mut self, k: &YamlValue) -> Option<&mut YamlValue> {
        self.map.get_mut(k)
    }

    /// Gets the given key’s corresponding entry in the map for insertion and/or
    /// in-place manipulation.
    #[inline]
    pub fn entry(&mut self, k: YamlValue) -> Entry {
        match self.map.entry(k) {
            indexmap::map::Entry::Occupied(occupied) => Entry::Occupied(OccupiedEntry { occupied }),
            indexmap::map::Entry::Vacant(vacant) => Entry::Vacant(VacantEntry { vacant }),
        }
    }

    /// Removes and returns the value corresponding to the key from the map.
    #[inline]
    pub fn remove(&mut self, k: &YamlValue) -> Option<YamlValue> {
        self.map.remove(k)
    }

    /// Returns the maximum number of key-value pairs the map can hold without
    /// reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Returns the number of key-value pairs in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether the map is currently empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the map of all key-value pairs.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a double-ended iterator visiting all key-value pairs in order of
    /// insertion. Iterator element type is `(&'a YamlValue, &'a YamlValue)`.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            iter: self.map.iter(),
        }
    }

    /// Returns a double-ended iterator visiting all key-value pairs in order of
    /// insertion. Iterator element type is `(&'a YamlValue, &'a mut ValuE)`.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

impl From<serde_yaml::Mapping> for Mapping {
    fn from(value: serde_yaml::Mapping) -> Self {
        Mapping {
            map: value.iter()
                .map(|(k, v)| (YamlValue::new(k.clone()), YamlValue::new(v.clone())))
                .collect()
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Mapping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the kv pairs in a way that is not sensitive to their order.
        let mut xor = 0;
        for (k, v) in self {
            let mut hasher = DefaultHasher::new();
            k.hash(&mut hasher);
            v.hash(&mut hasher);
            xor ^= hasher.finish();
        }
        xor.hash(state);
    }
}

impl PartialOrd for Mapping {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut self_entries = Vec::from_iter(self);
        let mut other_entries = Vec::from_iter(other);

        // Sort in an arbitrary order that is consistent with YamlValue's PartialOrd
        // impl.
        fn total_cmp(a: &YamlValue, b: &YamlValue) -> Ordering {
            match (a, b) {
                (YamlValue::Null, YamlValue::Null) => Ordering::Equal,
                (YamlValue::Null, _) => Ordering::Less,
                (_, YamlValue::Null) => Ordering::Greater,

                (YamlValue::Bool(a), YamlValue::Bool(b)) => a.cmp(b),
                (YamlValue::Bool(_), _) => Ordering::Less,
                (_, YamlValue::Bool(_)) => Ordering::Greater,

                (YamlValue::Number(a), YamlValue::Number(b)) => a.cmp(b),
                (YamlValue::Number(_), _) => Ordering::Less,
                (_, YamlValue::Number(_)) => Ordering::Greater,

                (YamlValue::String(a), YamlValue::String(b)) => a.cmp(b),
                (YamlValue::String(_), _) => Ordering::Less,
                (_, YamlValue::String(_)) => Ordering::Greater,

                (YamlValue::Sequence(a), YamlValue::Sequence(b)) => iter_cmp_by(a, b, total_cmp),
                (YamlValue::Sequence(_), _) => Ordering::Less,
                (_, YamlValue::Sequence(_)) => Ordering::Greater,

                (YamlValue::Mapping(a), YamlValue::Mapping(b)) => {
                    iter_cmp_by(a, b, |(ak, av), (bk, bv)| {
                        total_cmp(ak, bk).then_with(|| total_cmp(av, bv))
                    })
                }
            }
        }

        fn iter_cmp_by<I, F>(this: I, other: I, mut cmp: F) -> Ordering
            where
                I: IntoIterator,
                F: FnMut(I::Item, I::Item) -> Ordering,
        {
            let mut this = this.into_iter();
            let mut other = other.into_iter();

            loop {
                let x = match this.next() {
                    None => {
                        if other.next().is_none() {
                            return Ordering::Equal;
                        } else {
                            return Ordering::Less;
                        }
                    }
                    Some(val) => val,
                };

                let y = match other.next() {
                    None => return Ordering::Greater,
                    Some(val) => val,
                };

                match cmp(x, y) {
                    Ordering::Equal => {}
                    non_eq => return non_eq,
                }
            }
        }

        // While sorting by map key, we get to assume that no two keys are
        // equal, otherwise they wouldn't both be in the map. This is not a safe
        // assumption outside of this situation.
        let total_cmp = |&(a, _): &_, &(b, _): &_| total_cmp(a, b);
        self_entries.sort_by(total_cmp);
        other_entries.sort_by(total_cmp);
        self_entries.partial_cmp(&other_entries)
    }
}

impl<'a> Index<&'a YamlValue> for Mapping {
    type Output = YamlValue;
    #[inline]
    fn index(&self, index: &'a YamlValue) -> &YamlValue {
        self.map.index(index)
    }
}

impl<'a> IndexMut<&'a YamlValue> for Mapping {
    #[inline]
    fn index_mut(&mut self, index: &'a YamlValue) -> &mut YamlValue {
        self.map.index_mut(index)
    }
}

impl Extend<(YamlValue, YamlValue)> for Mapping {
    #[inline]
    fn extend<I: IntoIterator<Item = (YamlValue, YamlValue)>>(&mut self, iter: I) {
        self.map.extend(iter);
    }
}

impl FromIterator<(YamlValue, YamlValue)> for Mapping {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (YamlValue, YamlValue)>>(iter: I) -> Self {
        Mapping {
            map: IndexMap::from_iter(iter),
        }
    }
}

macro_rules! delegate_iterator {
    (($name:ident $($generics:tt)*) => $item:ty) => {
        impl $($generics)* Iterator for $name $($generics)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl $($generics)* ExactSizeIterator for $name $($generics)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }
    }
}

/// Iterator over `&Mapping`.
pub struct Iter<'a> {
    iter: indexmap::map::Iter<'a, YamlValue, YamlValue>,
}

delegate_iterator!((Iter<'a>) => (&'a YamlValue, &'a YamlValue));

impl<'a> IntoIterator for &'a Mapping {
    type Item = (&'a YamlValue, &'a YamlValue);
    type IntoIter = Iter<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.map.iter(),
        }
    }
}

/// Iterator over `&mut serde_yaml::Mapping`.
pub struct IterMut<'a> {
    iter: indexmap::map::IterMut<'a, YamlValue, YamlValue>,
}

delegate_iterator!((IterMut<'a>) => (&'a YamlValue, &'a mut YamlValue));

impl<'a> IntoIterator for &'a mut Mapping {
    type Item = (&'a YamlValue, &'a mut YamlValue);
    type IntoIter = IterMut<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

/// Iterator over `serde_yaml::Mapping` by value.
pub struct IntoIter {
    iter: indexmap::map::IntoIter<YamlValue, YamlValue>,
}

delegate_iterator!((IntoIter) => (YamlValue, YamlValue));

impl IntoIterator for Mapping {
    type Item = (YamlValue, YamlValue);
    type IntoIter = IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

/// Entry for an existing key-value pair or a vacant location to insert one.
pub enum Entry<'a> {
    /// Existing slot with equivalent key.
    Occupied(OccupiedEntry<'a>),
    /// Vacant slot (no equivalent key in the map).
    Vacant(VacantEntry<'a>),
}

/// A view into an occupied entry in a [`Mapping`]. It is part of the [`Entry`]
/// enum.
pub struct OccupiedEntry<'a> {
    occupied: indexmap::map::OccupiedEntry<'a, YamlValue, YamlValue>,
}

/// A view into a vacant entry in a [`Mapping`]. It is part of the [`Entry`]
/// enum.
pub struct VacantEntry<'a> {
    vacant: indexmap::map::VacantEntry<'a, YamlValue, YamlValue>,
}

impl<'a> Entry<'a> {
    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &YamlValue {
        match self {
            Entry::Vacant(e) => e.key(),
            Entry::Occupied(e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: YamlValue) -> &'a mut YamlValue {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    pub fn or_insert_with<F>(self, default: F) -> &'a mut YamlValue
        where
            F: FnOnce() -> YamlValue,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
        where
            F: FnOnce(&mut YamlValue),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a> OccupiedEntry<'a> {
    /// Gets a reference to the key in the entry.
    #[inline]
    pub fn key(&self) -> &YamlValue {
        self.occupied.key()
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn get(&self) -> &YamlValue {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    #[inline]
    pub fn get_mut(&mut self) -> &mut YamlValue {
        self.occupied.get_mut()
    }

    /// Converts the entry into a mutable reference to its value.
    #[inline]
    pub fn into_mut(self) -> &'a mut YamlValue {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key, and returns
    /// the entry's old value.
    #[inline]
    pub fn insert(&mut self, value: YamlValue) -> YamlValue {
        self.occupied.insert(value)
    }

    /// Takes the value of the entry out of the map, and returns it.
    #[inline]
    pub fn remove(self) -> YamlValue {
        self.occupied.swap_remove()
    }
}

impl<'a> VacantEntry<'a> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    #[inline]
    pub fn key(&self) -> &YamlValue {
        self.vacant.key()
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a
    /// mutable reference to it.
    #[inline]
    pub fn insert(self, value: YamlValue) -> &'a mut YamlValue {
        self.vacant.insert(value)
    }
}

impl Serialize for Mapping {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map_serializer = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map_serializer.serialize_entry(k, v)?;
        }
        map_serializer.end()
    }
}

impl<'de> Deserialize<'de> for Mapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Mapping;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a YAML mapping")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
            {
                Ok(Mapping::new())
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut values = Mapping::new();
                while let Some((k, v)) = visitor.next_entry()? {
                    values.insert(k, v);
                }
                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}
