use colosseum::sync::Arena;

pub trait Serializable {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out);
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
