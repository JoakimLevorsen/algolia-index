use super::{Deserializable, Serializable};

impl Serializable for char {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        // We just use the u64 encoding defined further down
        (*self as u32).serialize(output);
    }
}

impl Deserializable for char {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, v) = u32::deserialize(input)?;
        let char = char::from_u32(v)?;
        Some((input, char))
    }
}

impl Serializable for f32 {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        for byte in self.to_be_bytes() {
            output(byte);
        }
    }
}

impl Deserializable for f32 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let bytes = [
            *input.get(0)?,
            *input.get(1)?,
            *input.get(2)?,
            *input.get(3)?,
        ];
        let num = Self::from_be_bytes(bytes);
        Some((&input[4..], num))
    }
}

impl Serializable for bool {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        output(if *self { 1 } else { 0 });
    }
}

impl Deserializable for bool {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let byte = input.get(0)?;
        Some((&input[1..], *byte != 0))
    }
}

impl Serializable for u64 {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        let mut input = *self;
        loop {
            // We get 7 lowest bits
            #[allow(clippy::cast_possible_truncation)]
            let to_encode = (input as u8) & 0b0111_1111;
            input >>= 7;
            // If input is now 0, this was the last significant byte, and none follow
            if input == 0 {
                output(to_encode);
                break;
            }
            // Theres a following bit
            output(to_encode | 0b1000_0000);
        }
    }
}

impl Deserializable for u64 {
    fn deserialize(mut input: &[u8]) -> Option<(&[u8], Self)> {
        let mut out = 0;
        let mut eaten = 0;

        // We eat all the bytes that start with 1
        while input.is_empty() == false && eaten < 9 {
            // We shift the 7 data bits forwards, so they're all in front
            let byte = input[0];
            let more_bytes = byte & 0b1000_0000 == 0b1000_0000;
            let byte = byte << 1;
            out >>= 7;
            // We make the byte u64, and shift the data aaaaall the way to the front
            let byte = u64::from(byte) << (64 - 8);
            out |= byte;

            input = &input[1..];
            eaten += 1;

            if more_bytes == false {
                break;
            }

            if eaten > 10 {
                return None;
            }
        }

        // This means there is one last byte with 1 bit of real data
        if eaten == 9 {
            out >>= 1;
            if *input.get(0)? == 1 {
                out |= 0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
            }
            input = &input[1..];
            Some((input, out))
        } else if eaten == 0 {
            None
        } else {
            // We need to remove the extra padding from the beginning
            let reset_shift = 64 - (eaten * 7);
            let out = out >> reset_shift;
            Some((input, out))
        }
    }
}

impl Serializable for usize {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        (*self as u64).serialize(output);
    }
}

impl Deserializable for usize {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, v) = u64::deserialize(input)?;
        let v: usize = v.try_into().ok()?;
        Some((input, v))
    }
}

impl Serializable for u32 {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        u64::from(*self).serialize(output);
    }
}

impl Deserializable for u32 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let (input, v) = u64::deserialize(input)?;
        let v: u32 = v.try_into().ok()?;
        Some((input, v))
    }
}

impl Serializable for u8 {
    fn serialize<Out: FnMut(u8)>(&self, output: &mut Out) {
        output(*self);
    }
}
impl Deserializable for u8 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let me = *input.get(0)?;
        Some((&input[1..], me))
    }
}

#[cfg(test)]
mod tests {
    use super::{Deserializable, Serializable};

    fn serialize_deserialize<T: Serializable + Deserializable + std::fmt::Debug>(input: &T) -> T {
        let mut bytes = Vec::new();
        input.serialize(&mut |input| bytes.push(input));
        let (remaining_bytes, output) = T::deserialize(&bytes[..]).unwrap();
        assert!(
            remaining_bytes.is_empty(),
            "Not all bytes consumed for {input:?}"
        );
        output
    }

    fn many_serialize_deserialize<T: Serializable + Deserializable + Eq + std::fmt::Debug>(
        input: &[T],
    ) {
        for item in input.iter() {
            let deserialized = serialize_deserialize(item);
            assert!(
                item == &deserialized,
                "{item:?} was serialized/deserialized to {deserialized:?}"
            );
        }
    }

    #[test]
    fn test_char_serialization() {
        many_serialize_deserialize(&['a', char::default(), '\0', 'æ', '👽']);
    }

    #[test]
    fn test_u64_serialization() {
        many_serialize_deserialize(&[
            0b1000_0000,
            0b0111_1111,
            0,
            u64::MAX,
            u64::from(u8::MAX),
            u64::from(u32::MAX),
        ]);
    }

    #[test]
    fn test_bool_serialization() {
        many_serialize_deserialize(&[true, false]);
    }

    #[test]
    fn test_string_serialization() {
        let items =
            ["", "Hello", "hello", " assadio👽 s da sad👽i oasid\n\t\n"].map(str::to_string);
        many_serialize_deserialize(&items);
    }
}
