use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::entity::EntityType;

/// This structure represents ActivityPub Attachment.
/// Attachment comes in many forms, e.g. it could be `PropertyValue`.
#[derive(Deserialize, Serialize, Clone)]
pub struct Attachment {
    /// Type of attachment.
    #[serde(rename = "type")]
    pub object_type: EntityType,

    /// Content associated with attachment.
    #[serde(alias = "value")]
    pub content: Option<String>,

    /// Name of attachment, e.g. property name.
    pub name: Option<String>,

    /// URL to attachment content.
    #[serde(alias = "href")]
    pub url: Option<url::Url>,

    /// Media type of attachment, e.g. image/jpeg.
    #[serde(alias = "mediaType")]
    pub media_type: Option<String>,
}

/// Debug trait implementation to make attachment logged in a bit more readable form.
impl Debug for Attachment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ap::Attachment")
            .field("type", &self.object_type)
            .field("content", &self.content.as_deref()
                .unwrap_or("")
            )
            .field("name", &self.name.as_deref()
                .unwrap_or("")
            )
            .field("url", &self.url.as_ref()
                .map(|x| x.as_str())
                .unwrap_or("")
            )
            .field("media_type", &self.media_type.as_deref()
                .unwrap_or("")
            )
            .finish()
    }
}

/// Helper to wrap single or multiple attachments.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum AttachmentReference {
    Single(Attachment),
    List(Vec<Attachment>),
}

impl AttachmentReference {
    /// Returns vector with references to nested attachments.
    pub fn as_vec(&self) -> Vec<&Attachment> {
        match self {
            AttachmentReference::Single(attachment) => vec![attachment],
            AttachmentReference::List(attachments) => attachments.iter().collect()
        }
    }

    /// Consumes self and returns vector of attachments.
    pub fn into_vec(self) -> Vec<Attachment> {
        match self {
            AttachmentReference::Single(attachment) => vec![attachment],
            AttachmentReference::List(attachments) => attachments,
        }
    }
}
