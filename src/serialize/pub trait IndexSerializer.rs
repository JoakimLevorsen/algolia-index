pub trait IndexSerializer
where
    Self: Sized,
{
    type ArenaType;

    fn index_serialize(&self, output: &mut Vec<u8>);

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)>;

    fn deserialize_arena<'arena, 'input>(
        input: &'input [u8],
        arena: &'arena Self::ArenaType,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input;
}

impl IndexSerializer for char {
    type ArenaType = Arena<char>;

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

    fn deserialize_arena<'arena, 'input>(
        input: &'input [u8],
        arena: &'arena Self::ArenaType,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, char) = Self::deserialize(input)?;
        Some((input, arena.alloc(char)))
    }
}

impl IndexSerializer for f32 {
    fn index_serialize(&self, output: &mut Vec<u8>) {
        for byte in self.to_be_bytes() {
            output.push(byte)
        }
    }

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut bytes = [0; 4];
        for i in 0..4 {
            bytes[i] = *input.get(i)?
        }
        let num = Self::from_be_bytes(bytes);
        Some((&input[4..], num))
    }

    fn deserialize_arena<'arena, 'input>(
        input: &'input [u8],
        arena: &'arena Self::ArenaType,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, char) = Self::deserialize(input)?;
        Some((input, arena.alloc(char)))
    }

    type ArenaType = Arena<Self>;
}

impl IndexSerializer for u64 {
    type ArenaType = Arena<u64>;

    fn index_serialize(&self, output: &mut Vec<u8>) {
        for byte in self.to_be_bytes() {
            output.push(byte)
        }
    }

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut bytes = [0; 8];
        for i in 0..8 {
            bytes[i] = *input.get(i)?
        }
        let num = Self::from_be_bytes(bytes);
        Some((&input[4..], num))
    }

    fn deserialize_arena<'arena, 'input>(
        input: &'input [u8],
        arena: &'arena Self::ArenaType,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, char) = Self::deserialize(input)?;
        Some((input, arena.alloc(char)))
    }
}

impl<T: IndexSerializer> IndexSerializer for Vec<&T> {
    type ArenaType = Arena<T>;

    fn index_serialize(&self, output: &mut Vec<u8>) {
        (self.len() as u64).index_serialize(output);
        for item in self.iter() {
            item.index_serialize(output)
        }
    }

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        panic!("Unimplementable");
    }

    fn deserialize_arena<'arena, 'input>(
        input: &'input [u8],
        arena: &'arena Self::ArenaType,
    ) -> Option<(&'input [u8], Self)>
    where
        'arena: 'input,
    {
        let (mut input, len): (&'input [u8], u64) = u64::deserialize(input)?;
        let len = usize::try_from(len).unwrap();
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            let (next_input, item) = T::deserialize(input)?;
            let alloc = arena.alloc(item);
            out.push(&*alloc);
            input = next_input;
        }
        Some((input, &out))
    }
}

impl<'a, G: GramAtom> IndexSerializer for GramNode<'a, G> {
    type ArenaType = Arena<Self>;

    fn index_serialize(&self, output: &mut Vec<u8>) {
        self.item.index_serialize(output);
        self.weight.index_serialize(output);
        self.by_occurances.index_serialize(output);
    }

    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        todo!()
    }

    fn deserialize_arena<'arena, 'input>(
        input: &'input [u8],
        arena: &'arena Self::ArenaType,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        todo!()
    }
}
