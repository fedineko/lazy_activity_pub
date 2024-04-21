use std::collections::HashMap;

use serde::Deserialize;

use crate::actor::{CompoundActorReference, is_public_searchable_by};
use crate::attachment::AttachmentReference;
use crate::context::Context;
use crate::discoverable::{AllowReason, DenyReason, Discoverable};
use crate::entity::EntityType;
use crate::image::ImageReference;
use crate::object::{Object, ObjectTrait};
use crate::tag::TagReference;

/// This structure represents content, such as Note.
/// In other words: toots, blog posts and so on.
#[derive(Deserialize, Debug, Clone)]
pub struct Content {
    /// Properties inherited from ActivityPub Object.
    #[serde(flatten)]
    pub object_entity: Object,

    /// Author of content.
    #[serde(rename = "attributedTo")]
    pub attributed_to: CompoundActorReference,

    /// Is not really expected in Content object,
    /// but will be here just in case someone actually uses it.
    /// it is property to specify search indexing consent.
    /// See: <https://codeberg.org/fediverse/fep/src/branch/main/fep/5feb/fep-5feb.md>
    pub indexable: Option<bool>,

    /// Is not really expected in Content object,
    /// but will be here just in case someone actually uses it.
    /// Older way to specify discoverability flag that was for accounts/Actors,
    /// but somehow became a common thing for content as well.
    pub discoverable: Option<bool>,

    /// Point of time this content was published.
    pub published: chrono::DateTime<chrono::Utc>,

    /// Flag to indicate that content is sensitive.
    pub sensitive: Option<bool>,

    /// Fedibird's extension applicable to posts as well as to actors, indicates actor's
    /// consent for indexing servers to index this content.
    /// See: <https://github.com/mastodon/mastodon/pull/23808#issuecomment-1543273137>
    ///
    /// ```json
    /// "@context": [
    ///  "fedibird": "http://fedibird.com/ns#",
    ///  ...
    ///  "searchableBy": {
    ///     "@id": "fedibird:searchableBy",
    ///     "@type": "@id"
    ///  }
    /// ]
    ///```
    /// Example:
    /// ```json
    /// "searchableBy": [
    ///     "https://www.w3.org/ns/activitystreams#Public"
    /// ]
    /// ```
    #[serde(rename = "searchableBy")]
    pub searchable_by: Option<Vec<url::Url>>,

    /// Content summary, e.g. title or snippet of content.
    pub summary: Option<String>,

    /// Full content data.
    pub content: Option<String>,

    /// If localized into multiple languages, those will be part of `content_map`.
    /// Often - but not always - could be used to guess language of content.
    #[serde(rename = "contentMap")]
    pub content_map: Option<ContentMap>,

    /// Associated with content tags, including `#hastags`, mentions and emojis.
    pub tag: Option<TagReference>,
    pub attachment: Option<AttachmentReference>,

    /// Image associated with content.
    #[serde(alias = "image")]
    pub icon: Option<ImageReference>,
}

impl ObjectTrait for Content {
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

/// Helper struct to deal with different types of `contentMap` definition.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ContentMap {
    /// A proper dictionary of languages mapped to content.
    Map(HashMap<String, String>),
    /// Rarely observed `contentMap` variant that is just an JSON array with single item.
    List(Vec<String>),
}

impl ContentMap {
    /// Returns ContentMap as map regardless of actual underlying type.
    pub fn as_map(&self) -> HashMap<&str, &String> {
        match self {
            ContentMap::Map(map) => map.iter()
                .map(|(k, v)| (k.as_str(), v))
                .collect(),

            ContentMap::List(list) => {
                if list.is_empty() {
                    return HashMap::new();
                }

                HashMap::from([("default", list.first().unwrap())])
            }
        }
    }
}

impl Content {
    /// Returns content map for Content as language to content mapping.
    /// Content values are cleaned and joined with summary if any.
    /// `cleaner` function is applied to content before wrapping it into
    /// returned value.
    pub fn get_content_map(&self, cleaner: &dyn Fn(&str) -> String) -> Option<HashMap<String, String>> {
        if let Some(content_map) = &self.content_map {
            let summary = self.summary.as_ref();

            let map: HashMap<String, String> = content_map.as_map()
                .into_iter()
                .map(|(k, v)|
                    {
                        (
                            k.to_string(),
                            match summary {
                                None => cleaner(v),
                                Some(summary) => {
                                    let joined_value = format!("<p>{summary}</p>\n{v}");
                                    cleaner(&joined_value)
                                }
                            }
                        )
                    }
                )
                .collect();

            if !map.is_empty() {
                return Some(map);
            }
        }

        let summary = self.summary.as_ref();

        let content = self.content.as_ref()
            .or(self.object_entity.name.as_ref());

        if content.is_some() && summary.is_some() {
            let text = format!("<p>{}</p>\n{}", summary.unwrap(), content.unwrap());

            return Some(HashMap::from([
                (
                    "default".to_string(),
                    cleaner(&text)
                )
            ]));
        } else if summary.is_some() {
            return Some(HashMap::from([
                (
                    "default".to_string(),
                    cleaner(summary.unwrap())
                )
            ]));
        } else if let Some(content) = content{
            return Some(HashMap::from([
                (
                    "default".to_string(),
                    cleaner(content)
                )
            ]));
        }

        None
    }

    pub fn get_discoverable_state(&self, default_state: Discoverable) -> Discoverable {

        // `searchableBy` takes priority over other fields (which are usually account level fields)
        // See more on `searchableBy`:
        //   <https://github.com/mastodon/mastodon/pull/23808#issuecomment-1543273137>
        if let Some(searchable_by) = &self.searchable_by {
            if let Some(reason) = is_public_searchable_by(searchable_by) {
                return reason;
            }
        }

        // if 'indexable' is set, abid to it
        if let Some(indexable) = self.indexable {
            return match indexable {
                true => Discoverable::Allowed(AllowReason::Indexable),
                false => Discoverable::Denied(DenyReason::Indexable),
            };
        }

        // if 'discoverable' is set, but 'indexable' is not, then assume
        // 'discoverable' broadcasts the same intention of allowing/denying indexing
        // as older instances had 'discoverable' flag only.
        if let Some(discoverable) = self.discoverable {
            return match discoverable {
                true => Discoverable::Allowed(AllowReason::Discoverable),
                false => Discoverable::Denied(DenyReason::Discoverable),
            };
        }

        // otherwise assume default indexing option
        default_state
    }

    /// Returns discoverability state of content when checking opt-in state.
    ///
    /// This method is invoked when actor explicitly denies indexing.
    /// However, in case service supports opt-in per content item, this
    /// method validates actual discoverability state.
    pub fn get_optin_discoverable_state(&self) -> Discoverable {
        self.get_discoverable_state(Discoverable::Denied(DenyReason::Default))
    }

    /// Returns discoverability state of content when checking opt-out state.
    ///
    /// This method is invoked when actor explicitly allows indexing.
    /// However, in case service supports opt-out per content item, this
    /// method validates actual discoverability state.
    pub fn get_optout_discoverable_state(&self) -> Discoverable {
        self.get_discoverable_state(Discoverable::Allowed(AllowReason::Assumed))
    }
}


#[cfg(test)]
mod tests {
    use language_utils::content_cleaner::clean_some_content;

    use crate::content::Content;

    #[test]
    fn test_object_deserialize_success() {
        let serialized = r#"{
    "id": "https://live-theater.net/notes/xxxxxx",
        "type": "Note",
        "attributedTo": "https://live-theater.net/users/yyyyyyy",
        "content": "<p><i>\u200b:arisa_fuo_1:\u200b</i><span> xyz<br></span><a href=\"https://live-theater.net/play/zzzzzzz\">https://live-theater.net/play/zzzzzzz</a></p>",
        "_misskey_content": "$[rainbow :arisa_fuo_1:] xyz\nhttps://live-theater.net/play/zzzzzzz",
        "source": {
            "content": "$[rainbow :arisa_fuo_1:] xyz\nhttps://live-theater.net/play/zzzzzzz",
            "mediaType": "text/x.misskeymarkdown"
        },
        "published": "2024-01-01T01:01:01.000Z",
        "to": [
            "https://www.w3.org/ns/activitystreams#Public"
        ],
        "cc": [
            "https://live-theater.net/users/yyyyyyy/followers"
        ],
        "inReplyTo": null,
        "attachment": [],
        "sensitive": false,
        "tag": [
            {
                "id": "https://live-theater.net/emojis/arisa_fuo_1",
                "type": "Emoji",
                "name": ":arisa_fuo_1:",
                "updated": "2023-10-10T10:10:10.010Z",
                "icon": {
                    "type": "Image",
                    "mediaType": "image/png",
                    "url": "https://cdn.live-theater.net/null/webpublic-id.png"
                }
            }
        ]
    }"#;
        assert!(serde_json::from_str::<Content>(serialized).is_ok());
    }

    #[test]
    fn test_urls_are_kept_intact_in_content() {
        let serialized = r#"{
          "@context": [],
          "id": "https://z.y.x/users/xyz/statuses/123456789",
          "type": "Note",
          "summary": null,
          "inReplyTo": null,
          "published": "2024-01-22T13:37:04Z",
          "url": "https://z.y.x/users/xyz/statuses/123456789",
          "attributedTo": "https://z.y.x/users/xyz",
          "sensitive": false,
          "content": "Text<\u003c>br /\u003e\u003ca href=\"https://www.xyz.net/x/y/z/\" target=\"_blank\" rel=\"nofollow noopener noreferrer\" translate=\"no\"\u003e",
          "contentMap": {
            "en": "Text<\u003c>br /\u003e\u003ca href=\"https://www.xyz.net/x/y/z/\" target=\"_blank\" rel=\"nofollow noopener noreferrer\" translate=\"no\"\u003e"
          },
          "attachment": [],
          "tag": []
        }"#;

        let content: Content = serde_json::from_str(serialized).unwrap();
        let cleaner = |v: &str|  clean_some_content(v, true);
        let content_map = content.get_content_map(&cleaner).unwrap();

        assert_eq!(
            content_map.get("en").unwrap(),
            "Text&lt;&lt;&gt;br /&gt;<a href=\"https://www.xyz.net/x/y/z/\" rel=\"noopener noreferrer\"></a>"
        )
    }
}