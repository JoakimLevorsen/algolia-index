use typed_arena::Arena;

pub trait Serializable {
    fn serialize(&self, output: &mut Vec<u8>);
}

pub trait Deserializable
where
    Self: Sized,
{
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)>;
}

pub trait DeserializableCollection
where
    Self: Sized,
{
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)>;
}

pub trait ArenaDeserializable<'arena, ArenaContent>
where
    Self: Sized,
{
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<ArenaContent>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input;
}

pub trait ArenaDeserializableCollection<'arena, ArenaContent>
where
    Self: Sized,
{
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena Arena<ArenaContent>,
    ) -> Option<(&'input [u8], Self)>
    where
        'arena: 'input;
}

// impl<'arena, T> ArenaDeserializable<'arena, T> for T
// where
//     T: Sized + Deserializable,
// {
//     fn deserialize_arena<'input>(
//         input: &'input [u8],
//         arena: &'arena Arena<Self>,
//     ) -> Option<(&'input [u8], &'arena Self)>
//     where
//         'arena: 'input,
//     {
//         let (input, item) = Self::deserialize(input)?;
//         Some((input, arena.alloc(item)))
//     }
// }
