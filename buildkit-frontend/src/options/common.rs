use std::collections::HashMap;
use std::fmt;

use buildkit_proto::moby::buildkit::v1::frontend::CacheOptionsEntry as CacheOptionsEntryProto;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CacheOptionsEntry {
    #[serde(rename = "Type")]
    pub cache_type: CacheType,

    #[serde(rename = "Attrs")]
    pub attrs: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Local,
    Registry,
    Inline,
}

impl CacheOptionsEntry {
    pub fn from_legacy_list<'de, D>(deserializer: D) -> Result<Vec<Self>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LegacyVisitor;

        impl<'de> Visitor<'de> for LegacyVisitor {
            type Value = Vec<String>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("sequence")
            }

            fn visit_seq<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: SeqAccess<'de>,
            {
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(map))
            }
        }

        let legacy_refs = deserializer.deserialize_seq(LegacyVisitor)?;
        let new_refs_iter = legacy_refs.into_iter().map(|reference| CacheOptionsEntry {
            cache_type: CacheType::Registry,
            attrs: vec![(String::from("ref"), reference)].into_iter().collect(),
        });

        Ok(new_refs_iter.collect())
    }
}

impl Into<CacheOptionsEntryProto> for CacheOptionsEntry {
    fn into(self) -> CacheOptionsEntryProto {
        CacheOptionsEntryProto {
            r#type: self.cache_type.into(),
            attrs: self.attrs,
        }
    }
}

impl Into<String> for CacheType {
    fn into(self) -> String {
        match self {
            CacheType::Local => "local".into(),
            CacheType::Registry => "registry".into(),
            CacheType::Inline => "inline".into(),
        }
    }
}
