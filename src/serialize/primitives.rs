use super::{Deserializable, Serializable};

impl Serializable for char {
    fn serialize(&self, output: &mut Vec<u8>) {
        let num = *self as u64;
        num.serialize(output)
    }
}

impl Deserializable for char {
    fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
        u64::deserialize(input).map(|(input, v)| (input, char::from_u32(v as u32).unwrap()))
    }
}
// impl Serializable for char {
//     fn serialize(&self, output: &mut Vec<u8>) {
//         let mut buffer = [0; 4];
//         self.encode_utf8(&mut buffer);
//         for byte in buffer {
//             if byte == 0 {
//                 break;
//             }
//             output.push(byte)
//         }
//     }
// }

// impl Deserializable for char {
//     fn deserialize(input: &[u8]) -> Option<(&[u8], Self)> {
//         let mut n = 0;
//         for i in 0..4 {
//             n <<= 8;
//             n += *input.get(i)? as u32;
//             if let Some(char) = char::from_u32(n) {
//                 return Some((&input[(i + 1)..], char));
//             }
//         }
//         None
//     }
// }

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
        let mut bytes = [0u8; 8];
        for i in 0..8 {
            bytes[i] = *input.get(i)?;
        }
        let num = u64::from_be_bytes(bytes);
        Some((&input[8..], num))
    }
}
// impl Serializable for u64 {
//     fn serialize(&self, output: &mut Vec<u8>) {
//         let mut input = *self;
//         loop {
//             // We get 7 lowest bits
//             let to_encode = (input as u8) & 0b0111_1111;
//             input >>= 7;
//             // If input is now 0, this was the last significant byte, and none follow
//             if input == 0 {
//                 output.push(to_encode);
//                 break;
//             } else {
//                 // Theres a following bit
//                 output.push(to_encode | 0b1000_0000)
//             }
//         }
//     }
// }

// impl Deserializable for u64 {
//     fn deserialize(mut input: &[u8]) -> Option<(&[u8], Self)> {
//         let mut out = 0;
//         let mut eaten = 0;

//         // We eat all the bytes that start with 1
//         while input.len() > 0 && eaten < 9 {
//             // We shift the 7 data bits forwards, so they're all in front
//             let byte = input[0];
//             let more_bytes = byte & 0b1000_0000 == 0b1000_0000;
//             let byte = byte << 1;
//             out >>= 7;
//             // We make the byte u64, and shift the data aaaaall the way to the front
//             let byte = (byte as u64) << (64 - 8);
//             out |= byte;

//             input = &input[1..];
//             eaten += 1;

//             if more_bytes == false {
//                 break;
//             }

//             if eaten > 10 {
//                 return None;
//             }
//         }

//         // This means there is one last byte with 1 bit of real data
//         if eaten == 9 {
//             out >>= 1;
//             if *input.get(0)? == 1 {
//                 out |= 0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
//             }
//             input = &input[1..];
//             Some((input, out))
//         } else if eaten == 0 {
//             None
//         } else {
//             // We need to remove the extra padding from the beginning
//             let reset_shift = 64 - (eaten * 7);
//             let out = out >> reset_shift;
//             Some((input, out))
//         }
//     }
// }

#[test]
fn test_u64_serialization() {
    let to_test = [0b1000_0000, 0b0111_1111, 0, u64::MAX];
    for n in to_test {
        let mut bytes = Vec::new();
        n.serialize(&mut bytes);
        let (_, recovered) = u64::deserialize(&bytes[..]).unwrap();

        assert!(n == recovered, "Expected {n:064b} wasn't {recovered:064b}")
    }
}
