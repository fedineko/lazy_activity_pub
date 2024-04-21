use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// This enumeration represents entries in `@context` property.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ContextItem {
    /// Just a URL.
    Url(url::Url),

    /// Dictionary of strings mapped to any type.
    Mapping(HashMap<String, serde_json::Value>),
}

impl ContextItem {
    /// This method checks if context item matches given [url].
    /// Main purpose is to check existence of references to linked schemas.
    fn matches_url(&self, url: &url::Url) -> bool {
        match self {
            ContextItem::Url(item_url) => item_url == url,
            ContextItem::Mapping(_) => false
        }
    }

    /// This method checks for existence of [key] in context.
    /// Main purpose to quickly figure out declaration of properties.
    /// It is not quite reliable as property could be declared with
    /// a different name, but in the absence of full-fledged JSON-LD
    /// parser it works well enough.
    fn has_definition(&self, key: &str) -> bool {
        match self {
            ContextItem::Url(_) => false,
            ContextItem::Mapping(map) => map.contains_key(key)
        }
    }
}

/// This enumeration represents `@context` property.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Context {
    ContextItem(ContextItem),
    List(Vec<ContextItem>),
}

impl Context {
    /// This method checks if context item matches given `url`.
    /// Main purpose is to check existence of references to linked schemas.
    pub fn matches_url(&self, url: &url::Url) -> bool {
        match self {
            Context::ContextItem(item) => item.matches_url(url),
            Context::List(list) => list.iter()
                .any(|item| item.matches_url(url))
        }
    }

    /// This method checks for existence of key `name` in context.
    /// Main purpose to quickly figure out declaration of properties.
    /// It is not quite reliable as property could be declared with
    /// a different name, but in the absence of full-fledged JSON-LD
    /// parser it works well enough.
    pub fn has_definition(&self, name: &str) -> bool {
        match self {
            Context::ContextItem(item) => item.has_definition(name),
            Context::List(list) => list.iter()
                .any(|item| item.has_definition(name))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Context;

    #[test]
    fn test_context_deserialize_success() {
        let serialized = r#"[
            "https://www.w3.org/ns/activitystreams",
            {
                "ostatus": "http://ostatus.org#",
                "atomUri": "ostatus:atomUri",
                "inReplyToAtomUri": "ostatus:inReplyToAtomUri",
                "conversation": "ostatus:conversation",
                "sensitive": "as:sensitive",
                "toot": "http://joinmastodon.org/ns#",
                "votersCount": "toot:votersCount",
                "blurhash": "toot:blurhash",
                "focalPoint": {"@container": "@list", "@id": "toot:focalPoint"}
            }
            ]"#;
        assert!(serde_json::from_str::<Context>(serialized).is_ok());
    }

    #[test]
    fn test_search_in_context() {
        let serialized = r#"[
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1",
            {
              "manuallyApprovesFollowers": "as:manuallyApprovesFollowers",
              "toot": "http://joinmastodon.org/ns#",
              "discoverable": "toot:discoverable",
              "Device": "toot:Device",
              "messageType": "toot:messageType",
              "cipherText": "toot:cipherText",
              "suspended": "toot:suspended",
              "memorial": "toot:memorial",
              "indexable": "toot:indexable"
            }
          ]"#;

        let context: Context = serde_json::from_str(serialized).unwrap();

        assert!(context.has_definition("indexable"));
        assert!(context.has_definition("discoverable"));
        assert!(!context.has_definition("param"));
    }
}
