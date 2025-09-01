use serde::{Deserialize, Deserializer};
use serde::de::{self, SeqAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

/// Deserialize a field that can be either a single item or a vector of items
pub fn single_or_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct SingleOrVec<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for SingleOrVec<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("single item or array of items")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(item) = seq.next_element()? {
                vec.push(item);
            }
            Ok(vec)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            // If we get a single object, wrap it in a vector
            let item = T::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(vec![item])
        }
    }

    deserializer.deserialize_any(SingleOrVec(PhantomData))
}

/// Deserialize a field that can be either a single item or a vector, but can also be null/missing
pub fn single_or_vec_or_null<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SingleOrVecOrNull<T> {
        Null,
        Single(T),
        Multiple(Vec<T>),
    }

    match SingleOrVecOrNull::deserialize(deserializer)? {
        SingleOrVecOrNull::Null => Ok(None),
        SingleOrVecOrNull::Single(val) => Ok(Some(vec![val])),
        SingleOrVecOrNull::Multiple(vec) => Ok(Some(vec)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Item {
        id: String,
        name: String,
    }

    #[derive(Debug, Deserialize)]
    struct Container {
        #[serde(deserialize_with = "single_or_vec")]
        items: Vec<Item>,
    }

    #[test]
    fn test_single_item() {
        let json = r#"{"items": {"id": "1", "name": "Item 1"}}"#;
        let container: Container = serde_json::from_str(json).unwrap();
        assert_eq!(container.items.len(), 1);
        assert_eq!(container.items[0].id, "1");
    }

    #[test]
    fn test_multiple_items() {
        let json = r#"{"items": [{"id": "1", "name": "Item 1"}, {"id": "2", "name": "Item 2"}]}"#;
        let container: Container = serde_json::from_str(json).unwrap();
        assert_eq!(container.items.len(), 2);
        assert_eq!(container.items[0].id, "1");
        assert_eq!(container.items[1].id, "2");
    }
}