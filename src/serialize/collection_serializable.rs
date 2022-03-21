use typed_arena::Arena;

use crate::{ngramindex::GramAtom, ngramindex2::GramNode};

use super::{ItemArenalessSerialization, ItemSerializable};

pub trait CollectionSerializable
where
    Self: Sized,
{
    fn index_serialize(&self, output: &mut Vec<u8>);
}

impl<'a, T: ItemSerializable<'a>> CollectionSerializable for Vec<&T> {
    fn index_serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).index_serialize(output);
        for item in self.iter() {
            item.index_serialize(output)
        }
    }
}

pub trait CollectionDeSerializable<'arena, T>
where
    Self: Sized,
{
    type ArenaReturn;

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)>;

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<T>,
    ) -> Option<(&'input [u8], Self::ArenaReturn)>
    where
        'arena: 'input;
}

impl<'arena, T: ItemArenalessSerialization + ItemSerializable<'arena>>
    CollectionDeSerializable<'arena, T> for Vec<T>
where
    T: 'arena,
{
    type ArenaReturn = Vec<&'arena T>;

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (mut input, len) = u64::deserialize(input)?;
        let len = usize::try_from(len).unwrap();
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (next_input, item) = T::deserialize(input)?;
            out.push(item);
            input = next_input;
        }
        Some((input, out))
    }

    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<T>,
    ) -> Option<(&'input [u8], Self::ArenaReturn)>
    where
        'arena: 'input,
    {
        let (mut input, len) = u64::deserialize(input)?;
        let len = usize::try_from(len).unwrap();
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (next_input, item) = T::deserialize_arena(input, arena)?;
            out.push(item);
            input = next_input;
        }
        Some((input, out))
    }
}
