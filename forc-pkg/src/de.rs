use serde::de::{self, Visitor};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

/// Deserialize a map with unique keys.
///
/// It is a thin wrapper that will return an error if the map contains duplicate
/// keys. The value that is deserialized as an optional BTreeMap with the key
/// type String and the value type T.
///
/// This struct should be use through `deserialize_optional_btree` function.
/// Additionally, when using the derive macro the default option must be used
/// otherwise when the value is not present it will return an error.
#[derive(Debug, PartialEq)]
struct DuplicateKeyVisitor<'de, T: Deserialize<'de>>(std::marker::PhantomData<&'de T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for DuplicateKeyVisitor<'de, T> {
    type Value = Option<BTreeMap<String, T>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an object with unique keys or null")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let mut result = BTreeMap::new();

        while let Some((key, value)) = map.next_entry::<String, T>()? {
            if result.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate '{}'", key)));
            }
            result.insert(key, value);
        }

        Ok(Some(result))
    }
}

/// Deserialize a map with unique keys. If the map has the same key defined
/// multiple times an error will be return and the parsing will be aborted.
///
/// This function is a thin wrapper around the `DuplicateKeyVisitor` struct.
#[inline]
pub fn deserialize_optional_btree<'de, D, T>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, T>>, D::Error>
where
    D: de::Deserializer<'de>,
    T: Deserialize<'de> + 'de,
{
    deserializer.deserialize_map(DuplicateKeyVisitor(std::marker::PhantomData))
}
