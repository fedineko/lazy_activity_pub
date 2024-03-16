use serde::{Deserialize, Serialize};
use crate::entity::{Entity, EntityType};
use crate::image::ImageReference;

/// This structure is used to deserialize Tag object.
/// Despite its name that way, it could store mentions,
/// tags, emojis and whatnot.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
    /// Embedded [Entity] properties.
    /// Only entity type is interesting, maybe needs direct embedding.
    #[serde(flatten)]
    pub entity: Entity,

    /// Reference to Tag related details, e.g. list of posts for this tag.
    #[serde(alias = "href")]
    pub id: Option<url::Url>,

    /// Name of tag, e.g. `#tag`.
    #[serde(alias = "tag")]
    pub name: Option<String>,

    #[cfg(feature = "more_properties")]
    pub url: Option<url::Url>,

    /// For `Emoji` references image to display for emoji.
    #[serde(alias = "image")]
    pub icon: Option<ImageReference>,
}

impl Tag {
    pub fn object_id(&self) -> Option<&url::Url> {
        self.id.as_ref()
    }

    pub fn entity_type(&self) -> EntityType {
        self.entity.object_type
    }
}

/// Helper enumeration to wrap different variants of `tag` property.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TagReference {
    /// Single tag
    Single(Box<Tag>),

    /// Multiple tags
    List(Vec<Tag>),
}

impl TagReference {
    /// This function returns underlying data as vector of [Tag] objects.
    pub fn as_vec(&self) -> Vec<&Tag> {
        match self {
            TagReference::Single(tag) => vec![tag],
            TagReference::List(tags) => tags.iter().collect()
        }
    }
}