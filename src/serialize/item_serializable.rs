use std::{cell::RefCell, collections::HashMap, rc::Rc};

use typed_arena::Arena;

use crate::{ngramindex::GramAtom, ngramindex2::GramNode};

use super::{CollectionDeSerializable, CollectionSerializable};

pub trait ItemSerializable<'arena>
where
    Self: Sized,
{
    fn index_serialize(&self, output: &mut Vec<u8>);

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Self>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input;
}

pub trait ItemArenalessSerialization
where
    Self: Sized,
{
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)>;
}

impl ItemArenalessSerialization for char {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut n = 0;
        for i in 0..4 {
            n <<= 8;
            n += *input.get(i)? as u32;
            if let Some(char) = char::from_u32(n) {
                return Some((&input[(i + 1)..], char));
            }
        }
        None
    }
}

impl<'arena> ItemSerializable<'arena> for char {
    fn index_serialize(&self, output: &mut Vec<u8>) {
        let mut buffer = [0; 4];
        self.encode_utf8(&mut buffer);
        for byte in buffer {
            if byte == 0 {
                break;
            }
            output.push(byte)
        }
    }

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Self>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, char) = Self::deserialize(input)?;
        Some((input, arena.alloc(char)))
    }
}

impl ItemArenalessSerialization for f32 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut bytes = [0; 4];
        for i in 0..4 {
            bytes[i] = *input.get(i)?
        }
        let num = Self::from_be_bytes(bytes);
        Some((&input[4..], num))
    }
}

impl<'arena> ItemSerializable<'arena> for f32 {
    fn index_serialize(&self, output: &mut Vec<u8>) {
        for byte in self.to_be_bytes() {
            output.push(byte)
        }
    }

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Self>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, char) = Self::deserialize(input)?;
        Some((input, arena.alloc(char)))
    }
}

impl ItemArenalessSerialization for u64 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut bytes = [0; 8];
        for i in 0..8 {
            bytes[i] = *input.get(i)?
        }
        let num = Self::from_be_bytes(bytes);
        Some((&input[4..], num))
    }
}

impl<'arena> ItemSerializable<'arena> for u64 {
    fn index_serialize(&self, output: &mut Vec<u8>) {
        for byte in self.to_be_bytes() {
            output.push(byte)
        }
    }

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Self>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, char) = Self::deserialize(input)?;
        Some((input, arena.alloc(char)))
    }
}

impl<'arena, G: GramAtom> ItemArenalessSerialization for GramNode<'arena, G> {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        todo!()
    }
}

impl<'arena, G: GramAtom> ItemSerializable<'arena> for GramNode<'arena, G> {
    fn index_serialize(&self, output: &mut Vec<u8>) {
        self.item.index_serialize(output);
        self.weight.index_serialize(output);
        self.by_occurances.index_serialize(output);
    }

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<Self>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, item) = <G as ItemArenalessSerialization>::deserialize(input)?;
        let (input, weight) = f32::deserialize(input)?;
        let (input, by_occurances) = Vec::deserialize_arena(input, arena)?;
        let items = by_occurances.iter().fold(HashMap::new(), |mut map, item| {
            map.insert(item.item, *item);
            map
        });

        let node: &'arena GramNode<'arena, G> = arena.alloc(GramNode {
            item,
            weight,
            items,
            by_occurances,
        });
        Some((input, &*node))
    }
}
