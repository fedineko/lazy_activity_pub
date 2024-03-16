use std::fmt;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::context::{Context, ContextItem};


/// ActivityPub entity type
/// It mixes all sorts of entities together:
/// * Activities
/// * Actors
/// * Content entities
/// * Collections related
/// * etc.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum EntityType {
    // Activities
    Accept,
    Announce,
    Create,
    Delete,
    Follow,
    Reject,
    Undo,
    Update,

    // Actors
    Actor,
    Application,
    Organization,
    Group,
    Person,
    Service,

    // Collections
    Collection,
    CollectionPage,
    OrderedCollection,
    OrderedCollectionPage,

    // Content
    Article,
    Image,
    Link,
    Movie,
    Note,
    Page,
    Poll,
    Question,
    Tombstone,
    Video,

    // tags
    Emoji,
    Hashtag,
    Tag,
    Mention,

    PropertyValue,

    // IdentityProof,
    // Example:
    // {
    // "type": "IdentityProof",
    // "name": "xxx",
    // "signatureAlgorithm": "keybase",
    // "signatureValue": "abcdef"
    // }

    // TVSeason,
    // Example:
    // "tag": {
    //     "type": "TVSeason",
    //     "href": "https://neodb.social/tv/season/xxx",
    //     "image": "https://neodb.social/m/xxx",
    //     "name": "xxx"
    // },

    // attachments
    Document,

    // Unknown
    #[serde(other)]
    Unknown,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Checks if `entity_type` is one of supported content type options.
pub fn is_supported_content_type(entity_type: EntityType) -> bool {
    matches!(
        entity_type,
        EntityType::Note |
        EntityType::Video |
        EntityType::Movie |
        EntityType::Article
    )
}

/// The most basic ActivityPub data entity in this crate.
/// It is not actually part of spec but exists here for convenience.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Entity {
    /// Maps to `@context` property that defines schema of document.
    #[serde(rename = "@context")]
    pub context: Option<Context>,

    /// Type of ActivityPub object to figure out how to deal with it.
    #[serde(rename = "type")]
    pub object_type: EntityType,
}

impl Entity {
    /// Constructs new entity with specified `entity_type`.
    /// Is used quite rarely.
    pub fn new(entity_type: EntityType) -> Self {
        Self {
            context: Some(
                Context::List(vec![
                    ContextItem::Url(Url::parse("https://w3id.org/security/v1").unwrap()),
                    ContextItem::Url(Url::parse("https://www.w3.org/ns/activitystreams").unwrap()),
                ])
            ),
            object_type: entity_type,
        }
    }
}

/// Returns `true` if `entity_type` is one of supported actor types.
pub fn is_actor_type(entity_type: EntityType) -> bool {
    matches!(
        entity_type,
        EntityType::Actor |
        EntityType::Application |
        EntityType::Person |
        EntityType::Organization |
        EntityType::Service
    )
}

/// Converts string `value` to [EntityType] if it matches one of
/// supported content or actors types.
pub fn entity_type_from(value: &str) -> EntityType {
    match value {
        // Actors
        "Actor" => EntityType::Actor,
        "Application" => EntityType::Application,
        "Group" => EntityType::Group,
        "Organization" => EntityType::Organization,
        "Person" => EntityType::Person,
        "Service" => EntityType::Service,

        // Content
        "Article" => EntityType::Article,
        "Image" => EntityType::Image,
        "Movie" => EntityType::Page,
        "Note" => EntityType::Note,
        "Poll" => EntityType::Poll,
        "Question" => EntityType::Question,
        "Tombstone" => EntityType::Tombstone,
        "Video" => EntityType::Video,

        _ => EntityType::Unknown,
    }
}
