use super::{Deserializable, Serializable};

impl Serializable for char {
    fn serialize(&self, output: &mut Vec<u8>) {
        let mut buffer = [0; 4];
        self.encode_utf8(&mut buffer);
        for byte in buffer {
            if byte == 0 {
                break;
            }
            output.push(byte)
        }
    }
}

impl Deserializable for char {
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

impl Serializable for f32 {
    fn serialize(&self, output: &mut Vec<u8>) {
        for byte in self.to_be_bytes() {
            output.push(byte)
        }
    }
}

impl Deserializable for f32 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut bytes = [0; 4];
        for i in 0..4 {
            bytes[i] = *input.get(i)?
        }
        let num = Self::from_be_bytes(bytes);
        Some((&input[4..], num))
    }
}

impl Serializable for u64 {
    fn serialize(&self, output: &mut Vec<u8>) {
        for byte in self.to_be_bytes() {
            output.push(byte)
        }
    }
}

impl Deserializable for u64 {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        let mut bytes = [0; 8];
        for i in 0..8 {
            bytes[i] = *input.get(i)?
        }
        let num = Self::from_be_bytes(bytes);
        Some((&input[8..], num))
    }
}
