use crate::{
    ngram::HashExtractable,
    serialize::{ArenaDeserializable, Deserializable, Serializable},
};

#[derive(serde::Deserialize, PartialEq, Eq, Debug)]
pub struct Product {
    pub description: String,
    pub tags: Vec<String>,
    pub title: String,
    pub vendor: String,
    pub id: String,
}

impl PartialOrd for Product {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Product {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO: Improve
        self.id.cmp(&other.id)
    }
}

impl HashExtractable for &Product {
    type Inner = String;

    fn extract(&self) -> &Self::Inner {
        &self.id
    }
}

impl Serializable for &Product {
    fn serialize(&self, output: &mut Vec<u8>) {
        let Product {
            description,
            tags,
            title,
            vendor,
            id,
        } = self;
        for field in [description, title, vendor, id] {
            field.serialize(output)
        }
        tags.serialize(output);
    }
}

impl<'arena> ArenaDeserializable<'arena, Product> for Product {
    fn deserialize_arena<'input>(
        input: &'input [u8],
        arena: &'arena colosseum::sync::Arena<Product>,
    ) -> Option<(&'input [u8], &'arena Self)>
    where
        'arena: 'input,
    {
        let (input, description) = String::deserialize(input)?;
        let (input, title) = String::deserialize(input)?;
        let (input, vendor) = String::deserialize(input)?;
        let (input, id) = String::deserialize(input)?;
        let (input, tags) = Vec::deserialize(input)?;
        Some((
            input,
            arena.alloc(Product {
                description,
                tags,
                title,
                vendor,
                id,
            }),
        ))
    }
}
