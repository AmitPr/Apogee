use std::collections::HashMap;

use patricia_tree::PatriciaMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(with = "serialization")]
    pub routes: PatriciaMap<ServiceDescription>,
}

impl Config {
    pub fn route(&self, path: impl AsRef<str>) -> Option<&ServiceDescription> {
        let path = path.as_ref();
        let longest_prefix = self.routes.get_longest_common_prefix(path)?;
        let prefix = std::str::from_utf8(longest_prefix.0).unwrap();
        let remainder = path.strip_prefix(std::str::from_utf8(longest_prefix.0).unwrap())?;

        if !prefix.ends_with('/') && (!remainder.starts_with('/') && !remainder.is_empty()) {
            return None;
        }

        Some(longest_prefix.1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescription {
    pub name: String,
}

mod serialization {
    use super::*;
    use serde::de::{Deserializer, Visitor};
    use serde::ser::Serializer;
    use std::fmt;
    use std::marker::PhantomData;

    pub fn serialize<S>(
        map: &PatriciaMap<ServiceDescription>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = map.iter().collect::<HashMap<_, _>>();
        map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PatriciaMap<ServiceDescription>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatriciaMapVisitor(PhantomData<fn() -> PatriciaMap<ServiceDescription>>);
        impl<'de> Visitor<'de> for PatriciaMapVisitor {
            type Value = PatriciaMap<ServiceDescription>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of strings to strings")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut patricia_map = PatriciaMap::new();
                while let Some((key, value)) = map.next_entry::<String, ServiceDescription>()? {
                    let key = key.trim_end_matches('/');
                    patricia_map.insert(key, value);
                }
                Ok(patricia_map)
            }
        }

        deserializer.deserialize_map(PatriciaMapVisitor(PhantomData))
    }
}
