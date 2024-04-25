use std::fmt::{Debug, Formatter};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use crate::actor::CompoundActorReference;
use crate::actor::ActorReference::Url;
use crate::context::Context;
use crate::entity::{entity_type_from, EntityType};
use crate::object::{Object, ObjectReference, ObjectTrait};

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
    /// Creates new activity instance of type `activity_type` performed
    /// by `actor` with `object_reference` payload and `id` to identify
    /// activity.
    pub fn new(
        activity_type: EntityType,
        actor: url::Url,
        id: url::Url,
        object_reference: ObjectReference,
    ) -> Result<Self, serde_json::Error> {
        let object = serde_json::to_value(object_reference)?;

        Ok(Self {
            object_entity: Object::new_with_entity_type(activity_type, id),
            object,
            actor: CompoundActorReference::Reference(Url(actor)),
        })
    }

    /// Creates new activity instance from `object_entity` performed by `actor`
    /// with `object_reference` payload. `object_entity` provides ID and type
    /// activity.
    pub fn new_with_object_entity(
        object_entity: Object,
        actor: url::Url,
        object_reference: ObjectReference,
    ) -> Result<Self, serde_json::Error> {
        let object = serde_json::to_value(object_reference)?;

        Ok(Self {
            object_entity,
            object,
            actor: CompoundActorReference::Reference(Url(actor)),
        })
    }

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
    /// then returns empty Option.
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

    /// Returns true if `value` is string and matches pattern.
    fn value_matches_string(value: &serde_json::Value, pattern: &str) -> bool {
        value.as_str()
            .map(|s| s.contains(pattern))
            .unwrap_or(false)
    }

    /// Checks if value for 'to' property of activity or payload object
    /// matches string `pattern`.
    ///
    /// Logic here is quite... peculiar.
    pub fn to_field_matches(&self, pattern: &str) -> bool {

        // First, let's see if 'to' field on activity level matches pattern.
        if self.object_entity.matches(pattern) {
            return true;
        }

        // If payload is URL, assume it is 'to' value as a whole.
        // Quite a stretchy assumption, actually.
        if self.object.is_string() {
            return Self::value_matches_string(&self.object, pattern);
        }

        // Otherwise only objects are accepted
        if !self.object.is_object() {
            return false;
        }

        let to_value = match self.object.get("to") {
            None => return false,
            Some(value) => value
        };

        // Now need to figure out what type of that field is.
        if to_value.is_string() {
            return Self::value_matches_string(to_value, pattern);
        }

        // 'cc' or 'to' could be arrays of URLs
        if to_value.is_array() {
            return to_value.as_array()
                .map(|vec| vec.iter()
                    .any(|value| Self::value_matches_string(value, pattern))
                )
                .unwrap_or(false);
        }

        // Theoretically field could be an object, e.g. 'to' field could embed
        // actor object, in this case let's check its ID.
        if to_value.is_object() {
            return to_value.get("id")
                .map(|value| Self::value_matches_string(value, pattern))
                .unwrap_or(false);
        }

        false
    }

    /// Returns payload serialized to string.
    pub fn inner_object_as_string(&self) -> Option<String> {
        self.object.as_str()
            .map(|s| s.to_string())
    }
}

/// Represents basic Follow activity
pub struct FollowActivity {
    /// What object to follow.
    pub follow: url::Url,
    /// Activity ID.
    pub id: url::Url,
    /// Who intends to follow.
    pub by: url::Url,
}

impl Debug for FollowActivity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FollowActivity")
            .field("follow", &self.follow.as_str())
            .field("id", &self.id.as_str())
            .field("by", &self.by.as_str())
            .finish()
    }
}

impl FollowActivity {
    /// Creates Follow activity.
    ///
    /// - `follow` given actor or special URL to follow
    /// - `by` actor requesting following.
    /// - `host` is used to generate activity ID specific to host request
    ///         is sent to.
    pub fn new(
        follow: url::Url,
        host: &str,
        by: url::Url
    ) -> Result<Self, url::ParseError> {
        let follow_path = format!("{}/follow/{host}/public", by.path());
        let id = by.join(&follow_path)?;

        Ok(Self {
            follow,
            id,
            by,
        })
    }

    /// Converts this into [Activity] ready to be serialized and sent over wire.
    pub fn into_activity(self) -> Result<Activity, serde_json::Error> {
        Activity::new(
            EntityType::Follow,
            self.by,
            self.id,
            ObjectReference::Url(self.follow),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::activity::Activity;
    use crate::actor::PUBLIC_ADDRESSEE;

    const SERIALIZED_DATA: &str = r#" {
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
                "type": "Tombstone",
                "to": "https://1.2/3"
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

    #[test]
    fn test_tombstone_deserialize_success() {
        assert!(serde_json::from_str::<Activity>(SERIALIZED_DATA).is_ok());
    }

    #[test]
    fn test_addressee_field_matches() {
        let value = serde_json::from_str::<Activity>(SERIALIZED_DATA).unwrap();

        assert!(value.to_field_matches(PUBLIC_ADDRESSEE));

        assert!(
            value.to_field_matches(
                "https://1.2/3",
            )
        );
    }
}