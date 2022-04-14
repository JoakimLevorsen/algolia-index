use super::{Deserializable, Serializable};

pub fn serialize<T, I>(input: I, output: &mut Vec<u8>)
where
    T: std::ops::Sub<Output = T> + Ord + Serializable + Copy,
    I: Iterator<Item = T>,
{
    let mut content: Vec<T> = input.collect();
    content.sort();

    content.len().serialize(output);

    let mut previous = None;
    for item in content {
        let to_serialize = match previous {
            Some(previous) => item - previous,
            None => item,
        };

        to_serialize.serialize(output);
        previous = Some(item);
    }
}

pub fn deserialize<T>(input: &[u8]) -> Option<(&[u8], Vec<T>)>
where
    T: std::ops::Add<Output = T> + Ord + Serializable + Deserializable + Copy,
{
    let (mut input, len) = usize::deserialize(input)?;

    let mut output = Vec::with_capacity(len);
    let mut previous = None;
    for _ in 0..len {
        let (new_input, parsed) = T::deserialize(input)?;
        input = new_input;

        let item = match previous {
            Some(previous) => parsed + previous,
            None => parsed,
        };

        previous = Some(item);
        output.push(item);
    }

    Some((input, output))
}
