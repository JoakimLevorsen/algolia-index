use std::hash::Hash;

use ahash::AHashMap;
use colosseum::sync::Arena;

use super::{
    ArenaDeserializable, ArenaDeserializableCollection, Deserializable, DeserializableCollection,
    Serializable,
};

impl<T: Serializable> Serializable for Vec<&T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.len().serialize(output);
        for item in self.iter() {
            item.serialize(output)
        }
    }
}

impl<T: Serializable> Serializable for &Vec<T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.len().serialize(output);
        for item in self.iter() {
            item.serialize(output)
        }
    }
}

impl Serializable for Vec<usize> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.len().serialize(output);
        for item in self.iter() {
            item.serialize(output)
        }
    }
}

impl<T: Deserializable> Deserializable for Vec<T> {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = usize::deserialize(input)?;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (new_input, item) = T::deserialize(input)?;
            input = new_input;
            out.push(item)
        }
        Some((input, out))
    }
}

impl<'arena, T: ArenaDeserializable<'arena, T>> ArenaDeserializableCollection<'arena, T>
    for Vec<&'arena T>
{
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<T>,
    ) -> Option<(&'input [u8], Self)>
    where
        'arena: 'input,
    {
        let (mut input, len) = usize::deserialize(input)?;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (new_input, item) = T::deserialize_arena(input, arena)?;
            input = new_input;
            out.push(item)
        }
        Some((input, out))
    }
}

impl Serializable for &'_ str {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.len().serialize(output);
        for item in self.bytes() {
            output.push(item)
        }
    }
}

impl Serializable for String {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.as_str().serialize(output)
    }
}

fn limit_string_len(input: &str, max_len: usize) -> &str {
    let mut last_candidate = None;
    for (offset, char) in input.char_indices() {
        if offset > max_len {
            // Then we return either the last viable candidate, or now
            return match last_candidate {
                Some(viable) => &input[0..viable],
                None => &input[0..offset],
            };
        }
        if char == ' ' {
            last_candidate = Some(offset);
        }
    }
    input
}

pub fn serialize_string_with_limit(input: &str, max_len: usize, output: &mut Vec<u8>) {
    limit_string_len(input, max_len).serialize(output)
}

#[test]
fn test() {
    for str in [
        "",
        "asudfhj asiudf",
        "as ias ias is a",
        "oai aisæø øåæå",
        "👽 👽 👽👽 👽👽",
    ] {
        println!("'{}'", limit_string_len(str, 12))
    }
}

impl Deserializable for String {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, len) = usize::deserialize(input)?;
        let bytes = &input[..len];
        let string = String::from_utf8(bytes.to_vec()).ok()?;
        Some((&input[len..], string))
    }
}

impl<K: Serializable, V: Serializable> Serializable for AHashMap<K, V> {
    fn serialize(&self, output: &mut Vec<u8>) {
        self.len().serialize(output);
        for (key, value) in self.iter() {
            key.serialize(output);
            value.serialize(output)
        }
    }
}

impl<'arena, K, V> ArenaDeserializableCollection<'arena, V> for AHashMap<K, &'arena V>
where
    K: Deserializable + Eq + Hash,
    V: ArenaDeserializable<'arena, V>,
{
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<V>,
    ) -> Option<(&'input [u8], Self)>
    where
        'arena: 'input,
    {
        let (mut input, len) = usize::deserialize(input)?;
        let mut out = AHashMap::with_capacity(len);
        for _ in 0..len {
            let (new_input, key) = K::deserialize(input)?;
            let (new_input, value) = V::deserialize_arena(new_input, arena)?;
            input = new_input;
            out.insert(key, value);
        }
        Some((input, out))
    }
}

impl<K: Deserializable + Eq + Hash, V: Deserializable> Deserializable for AHashMap<K, V> {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = usize::deserialize(input)?;
        let mut out = AHashMap::with_capacity(len);
        for _ in 0..len {
            let (new_input, key) = K::deserialize(input)?;
            let (new_input, value) = V::deserialize(new_input)?;
            input = new_input;
            out.insert(key, value);
        }
        Some((input, out))
    }
}

impl<T: Serializable, const N: usize> Serializable for [T; N] {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).serialize(output);
        for item in self.iter() {
            item.serialize(output)
        }
    }
}

impl<T: Default + Deserializable + Copy, const N: usize> DeserializableCollection for [T; N] {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = usize::deserialize(input)?;
        assert!(len == N);
        let mut out = [T::default(); N];
        for position in out.iter_mut().take(len) {
            let (new_input, item) = T::deserialize(input)?;
            input = new_input;
            *position = item;
        }
        Some((input, out))
    }
}

impl<T: Default + Deserializable + Copy, const N: usize> Deserializable for [T; N] {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = usize::deserialize(input)?;
        assert!(len == N);
        let mut out = [T::default(); N];
        for position in out.iter_mut().take(len) {
            let (new_input, item) = T::deserialize(input)?;
            input = new_input;
            *position = item;
        }
        Some((input, out))
    }
}
