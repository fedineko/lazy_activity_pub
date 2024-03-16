use log::{debug, error};
use serde::{Deserialize, Serialize};
use crate::actor::CompoundActorReference;
use crate::context::Context;
use crate::entity::{entity_type_from, EntityType};
use crate::object::{Object, ObjectTrait};

/// Activity object.
/// See: <https://www.w3.org/TR/activitystreams-core/#activities>
#[derive(Deserialize, Serialize, Debug)]
pub struct Activity {
    /// Embeds essential properties from ActivityStreams Object
    #[serde(flatten)]
    pub object_entity: Object,

    /// Payload Object in form of serde_json's Value.
    pub object: serde_json::Value,

    /// Actor reference.
    pub actor: CompoundActorReference,
}

impl ObjectTrait for Activity {
    fn context(&self) -> Option<&Context> {
        self.object_entity.context()
    }

    fn object_id(&self) -> &url::Url {
        self.object_entity.object_id()
    }

    fn entity_type(&self) -> EntityType {
        self.object_entity.entity_type()
    }
}

impl Activity {
    /// Returns this activity ID.
    pub fn activity_id(&self) -> &url::Url {
        self.object_id()
    }

    /// Returns type of payload object.
    /// If payload value does not have `type` property
    /// then returns [EntityType::Unknown].
    pub fn inner_object_type(&self) -> EntityType {
        return if self.object.is_object() {
            self.object.get("type")
                .and_then(|value| value.as_str())
                .map(entity_type_from)
                .unwrap_or(EntityType::Unknown)
        } else {
            debug!("No 'type' field in object: {:?}", self.object);
            EntityType::Unknown
        };
    }

    /// Returns ID of payload object.
    /// If payload value does not have `id` property and is not string
    /// then returns empty Optional.
    pub fn inner_object_id(&self) -> Option<url::Url> {
        return if self.object.is_object() {
            self.object.get("id")
                .and_then(|value| value.as_str())
                .and_then(|s| url::Url::parse(s).ok())
        } else if self.object.is_string() {
            self.object.as_str()
                .and_then(|s| url::Url::parse(s).ok())
        } else {
            error!("Failed to get ID of inner object: {:?}", self.object);
            None
        };
    }

    /// Returns payload serialized to string.
    pub fn inner_object_as_string(&self) -> Option<String> {
        self.object.as_str()
            .map(|s| s.to_string())
    }
}


#[cfg(test)]
mod tests {
    use crate::activity::Activity;

    #[test]
    fn test_tombstone_deserialize_success() {
        let serialized = r#" {
            "@context": [
                "https://www.w3.org/ns/activitystreams",
                 {
                    "atomUri": "ostatus:atomUri",
                     "ostatus": "http://ostatus.org#"
                 }
            ],
            "actor": "https://mastodon.world/users/xxxxxxx",
            "id": "https://mastodon.world/users/xxxxxxx/statuses/12345678#delete",
            "object": {
                "atomUri": "https://mastodon.world/users/xxxxxxx/statuses/12345678",
                "id": "https://mastodon.world/users/xxxxxxx/statuses/12345678",
                "type": "Tombstone"
            },
            "signature": {
                "created": "2024-01-01T01:01:01Z",
                "creator": "https://mastodon.world/users/xxxxxxx#main-key",
                "signatureValue": "RSA==",
                "type": "RsaSignature2017"
            },
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "type": "Delete"
        }"#;

        assert!(serde_json::from_str::<Activity>(serialized).is_ok());
    }
}