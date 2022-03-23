use std::{collections::HashMap, hash::Hash};

use colosseum::sync::Arena;

use super::{
    ArenaDeserializable, ArenaDeserializableCollection, Deserializable, DeserializableCollection,
    Serializable,
};

// impl<T: Serializable> Serializable for Vec<T> {
//     fn serialize(&self, output: &mut Vec<u8>) {
//         (self.len() as u64).serialize(output);
//         for item in self.iter() {
//             item.serialize(output)
//         }
//     }
// }

impl<T: Serializable> Serializable for Vec<&T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).serialize(output);
        for item in self.iter() {
            item.serialize(output)
        }
    }
}

impl<T: Serializable> Serializable for &Vec<T> {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).serialize(output);
        for item in self.iter() {
            item.serialize(output)
        }
    }
}

impl<T: Deserializable> Deserializable for Vec<T> {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = u64::deserialize(input)?;
        let mut out = Vec::with_capacity(len.try_into().unwrap());
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
        let (mut input, len) = u64::deserialize(input)?;
        let mut out = Vec::with_capacity(match len.try_into() {
            Ok(v) => v,
            Err(e) => panic!(
                "Failed to convert {len} to usize with max of {}",
                usize::MAX
            ),
        });
        for _ in 0..len {
            let (new_input, item) = T::deserialize_arena(input, arena)?;
            input = new_input;
            out.push(item)
        }
        Some((input, out))
    }
}

impl Serializable for String {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).serialize(output);
        for item in self.bytes() {
            output.push(item)
        }
    }
}

impl Deserializable for String {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, len) = u64::deserialize(input)?;
        let len = len as usize;
        let bytes = &input[..len];
        let string = String::from_utf8(bytes.to_vec()).ok()?;
        Some((&input[len..], string))
    }
}

impl<K: Serializable, V: Serializable> Serializable for HashMap<K, V> {
    fn serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).serialize(output);
        for (key, value) in self.iter() {
            key.serialize(output);
            value.serialize(output)
        }
    }
}

impl<'arena, K, V> ArenaDeserializableCollection<'arena, V> for HashMap<K, &'arena V>
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
        let (mut input, len) = u64::deserialize(input)?;
        let mut out = HashMap::with_capacity(len.try_into().unwrap());
        for _ in 0..len {
            let (new_input, key) = K::deserialize(input)?;
            let (new_input, value) = V::deserialize_arena(new_input, arena)?;
            input = new_input;
            out.insert(key, value);
        }
        Some((input, out))
    }
}

pub fn manual_hashmap_deserialize<'arena, 'input, K, V, VCon>(
    input: &'input [u8],
    arena: &'arena Arena<V>,
) -> Option<HashMap<K, VCon>>
where
    'arena: 'input,
    K: Deserializable + Eq + Hash,
    VCon: ArenaDeserializableCollection<'arena, V>,
{
    let (mut input, len) = u64::deserialize(input)?;
    let mut out = HashMap::with_capacity(len.try_into().unwrap());
    for _ in 0..len {
        let (new_input, key) = K::deserialize(input)?;
        let (new_input, value) = VCon::deserialize_arena(new_input, arena)?;
        input = new_input;
        out.insert(key, value);
    }
    Some(out)
}

// impl<'arena, Deep, K, V> ArenaDeserializableCollection<'arena, Deep> for HashMap<K, V>
// where
//     K: Deserializable + Eq + Hash,
//     V: ArenaDeserializableCollection<'arena, Deep>,
// {
//     fn deserialize_arena<'input>(
//         input: &'input [u8],
//         arena: &'arena Arena<Deep>,
//     ) -> Option<(&'input [u8], Self)>
//     where
//         'arena: 'input,
//     {
//         let (mut input, len) = u64::deserialize(input)?;
//         let mut out = HashMap::with_capacity(len.try_into().unwrap());
//         for _ in 0..len {
//             let (input, key) = K::deserialize(input)?;
//             let (new_input, value) = V::deserialize_arena(input, arena)?;
//             input = new_input;
//             out.insert(key, value);
//         }
//         Some((input, out))
//     }
// }

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
        let (mut input, len) = u64::deserialize(input)?;
        let len: usize = len.try_into().unwrap();
        assert!(len == N);
        let mut out = [T::default(); N];
        for i in 0..len {
            let (new_input, item) = T::deserialize(input)?;
            input = new_input;
            out[i] = item;
        }
        Some((input, out))
    }
}

impl<T: Default + Deserializable + Copy, const N: usize> Deserializable for [T; N] {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = u64::deserialize(input)?;
        let len: usize = len.try_into().unwrap();
        assert!(len == N);
        let mut out = [T::default(); N];
        for i in 0..len {
            let (new_input, item) = T::deserialize(input)?;
            input = new_input;
            out[i] = item;
        }
        Some((input, out))
    }
}
