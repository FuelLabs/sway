use serde::de::{self, Visitor};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, PartialEq)]
pub(crate) struct DuplicateKeyVisitor<'de, T: Deserialize<'de>>(
    pub(crate) std::marker::PhantomData<&'de T>,
);

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
