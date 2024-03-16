use std::collections::HashMap;
use std::sync::OnceLock;
use log::warn;
use serde::{Deserialize, Serialize};
use crate::attachment::AttachmentReference;
use crate::context::Context;
use crate::discoverable::{AllowReason, DenyReason, Discoverable};
use crate::entity::EntityType;
use crate::image::ImageReference;
use crate::object::{Object, ObjectTrait};
use crate::tag::TagReference;

static SECURITY_CONTEXT: OnceLock<url::Url> = OnceLock::new();

/// Actor object with a few commonly used properties.
/// See: <https://www.w3.org/TR/activitypub/#actor-objects>
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Actor {
    /// Embeds essential properties from ActivityStreams Object
    #[serde(flatten)]
    pub object_entity: Object,

    /// Link to this actor's inbox.
    pub inbox: url::Url,

    /// URL of this actor's outbox.
    /// No use so far, so commented out.
    #[cfg(feature = "more_properties")]
    pub outbox: Option<url::Url>,

    /// Link to followers collection.
    /// SHOULD have as Actor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers: Option<url::Url>,

    /// Link to collection of actors this one follows.
    /// SHOULD have as Actor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<url::Url>,

    /// Link to collection of objects liked by this actor.
    /// MAY have as Actor
    #[cfg(feature = "more_properties")]
    pub liked: Option<url::Url>,

    /// Short username without any guarantees on uniqueness.
    /// Usually it is actual account name.
    #[serde(rename = "preferredUsername")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,

    /// A map of additional endpoints that Actor MAY share.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Endpoints>,

    /// Optional map to represent name in multiple languages,
    /// Comes from ActivityStream spec.
    /// See: <https://www.w3.org/TR/activitystreams-vocabulary/#dfn-name>
    #[serde(rename = "nameMap")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_map: Option<HashMap<String, String>>,

    /// Short summary of what this actor about, could have summaryMap as well,
    /// though ignored in this crate
    /// MAY have as ActivityStream inheritance.
    /// See: <https://www.w3.org/TR/activitystreams-vocabulary/#dfn-summary>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Optional image/icon used as Actor's avatar.
    /// See: <https://www.w3.org/TR/activitystreams-vocabulary/#dfn-icon>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<ImageReference>,

    /// MAY have as extension, in practice it is somewhat difficult to find
    /// Actor without it. Public key used to sign messages from Actor.
    #[serde(rename = "publicKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<PublicKeyReference>,

    /// Property to specify search indexing consent.
    /// See: <https://codeberg.org/fediverse/fep/src/branch/main/fep/5feb/fep-5feb.md>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexable: Option<bool>,

    /// Older way to specify discoverability flag that was for accounts/Actors,
    /// but somehow became a common thing for content as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,

    /// Fedibird's extension applicable to posts as well as to actors.
    /// For actors, it - more or less - duplicates fields above but allows scoped
    /// consent for specific content.
    /// See: <https://github.com/mastodon/mastodon/pull/23808#issuecomment-1543273137>
    #[serde(rename = "searchableBy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchable_by: Option<Vec<url::Url>>,

    /// Tags for this actor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<TagReference>,

    /// Any attachments, e.g. profile pictures, properties and similar stuff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AttachmentReference>,
}

impl ObjectTrait for Actor {
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

impl Actor {
    /// This method returns Actor's `name` property inherited from Object,
    /// if any.
    pub fn name(&self) -> &Option<String> {
        &self.object_entity.name
    }

    /// This function checks if Actor object has security context and public key
    /// related property. Returns reference to self, wrapped into Optional.
    /// It is empty Optional if validation did fail.
    pub fn validate_security_context(&self) -> Option<&Self> {
        let context = self.context();

        if context.is_none() {
            return Some(self);
        }

        let security_context_url = SECURITY_CONTEXT.get_or_init(|| {
            url::Url::parse("https://w3id.org/security/v1").unwrap()
        });

        if !context.unwrap().matches_url(security_context_url) {
            return Some(self);
        }

        if self.public_key.is_none() {
            warn!(
                "Person {} uses context {} but 'publicKeyPem' is not defined",
                security_context_url.as_str(), self.object_id()
            );

            return None;
        }

        Some(self)
    }

    /// Checks if Actor identifies itself as Person.
    pub fn is_person(&self) -> bool {
        matches!(self.entity_type(), EntityType::Person)
    }

    /// This function returns discoverability state for Actor.
    /// Multiple properties are checked, if nothing matches actor is assumed
    /// to be discoverable.
    pub fn get_discoverable_state(&self) -> Discoverable {
        //  Check for property values of account, this is more of escape hatch for
        //  services that do not support 'discoverable' and 'indexable' properties,
        //  yet do want to indicate opt-out or opt-in explicitly.
        //
        //  If there is 'fedineko:index' then abid to it.
        //  Permissive value is 'allow', everything else is treated as indexing is denied.
        //
        //  {
        //    "@context": [
        //        ...
        //        "PropertyValue": "schema:PropertyValue",
        //        "value": "schema:value",
        //        ...
        //    ],
        //    ...
        //    "attachment": [
        //      {
        //        "type": "PropertyValue",
        //        "name": "fedineko:index",
        //        "value": "deny"
        //      },
        //      ...
        //    ],
        //    ...
        //  }

        let attachments: Vec<_> = self.attachment.iter()
            .flat_map(|att| att.as_vec())
            .collect();

        for attachment in attachments.into_iter() {
            if attachment.object_type != EntityType::PropertyValue {
                continue;
            }

            if attachment.name.is_none() {
                continue;
            }

            if attachment.content.is_none() {
                continue;
            }

            match attachment.name.as_ref().unwrap().as_str() {
                "fedineko:index" => {}
                _ => continue
            }

            return match attachment.content.as_ref().unwrap().as_str() {
                "allow" => Discoverable::Allowed(AllowReason::FedinekoProperty),
                // anything else is assumed to be intention to deny indexing.
                _ => Discoverable::Denied(DenyReason::FedinekoProperty),
            };
        }

        // if there is inconsistency between indexable/discoverable and searchableBy,
        // the latter takes priority.
        if let Some(searchable_by) = &self.searchable_by {
            if let Some(reason) = is_public_searchable_by(searchable_by) {
                return reason;
            }
        }

        let context = match self.context() {
            // No context means object as a whole is not well-formed.
            // That is weird so let's deny indexing by default.
            None => {
                warn!(
                    "{} is not discoverable by default because it lacks required context",
                    self.object_id_str()
                );

                return Discoverable::Denied(DenyReason::Default);
            }
            Some(context) => context
        };

        // if 'indexable' is set, abid to it
        // See: <https://codeberg.org/fediverse/fep/src/branch/main/fep/5feb/fep-5feb.md>
        if context.has_definition("indexable") {
            return if let Some(indexable) = self.indexable {
                match indexable {
                    true => Discoverable::Allowed(AllowReason::Indexable),
                    false => Discoverable::Denied(DenyReason::Indexable),
                }
            } else {
                warn!(
                    "{} is not discoverable because 'indexable' is declared but not set",
                    self.object_id_str()
                );

                Discoverable::Denied(DenyReason::Indexable)
            };
        }

        // if 'discoverable' is set, but 'indexable' is not, then assume
        // 'discoverable' indicates the same intention to allow/deny indexing.
        // Some older instances have 'discoverable' flag only.
        // Historically it was for accounts only and for a slightly different purpose,
        // but is used for posts as well nowadays.
        if context.has_definition("discoverable") {
            if let Some(discoverable) = self.discoverable {
                return match discoverable {
                    true => Discoverable::Allowed(AllowReason::Discoverable),
                    false => Discoverable::Denied(DenyReason::Discoverable),
                };
            }

            // There is no clear spec for the case when it is not set but declared,
            // see Mastodon documentation:
            //  - <https://docs.joinmastodon.org/spec/activitypub/#discoverable>
            //  - <https://docs.joinmastodon.org/entities/Account/#discoverable>
            //
            // Assuming the conservative option: no indexing if not set.
            warn!(
                "{} is not discoverable because 'discoverable' is declared but not set",
                self.object_id_str()
            );

            return Discoverable::Denied(DenyReason::Discoverable);
        }

        // All deny options are exhausted, so assuming content from actor is indexable.
        // E.g. lemmy has no similar properties, it is not required by ActivityPub spec.
        // See: <https://www.w3.org/TR/activitypub/#actor-objects>

        warn!("{} is assumed to be discoverable", self.object_id_str());

        Discoverable::Allowed(AllowReason::Assumed)
    }
}

/// Helper enumeration to wrap different ways to refer actor into
/// one serializable entity.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ActorReference {
    /// Actor as defined in this module.
    Actor(Box<Actor>),
    /// Not quite actor, just ID and a few other Object properties.
    BasicData(Box<Object>),
    /// The simplest option - just a URL to Actor.
    Url(url::Url),
}

impl ActorReference {
    /// This function returns Actor ID.
    pub fn id(&self) -> &url::Url {
        match self {
            Self::Actor(actor) => actor.object_id(),
            Self::Url(url) => url,
            Self::BasicData(object) => object.object_id(),
        }
    }

    /// This function returns Actor's entity type if possible to get it
    /// from nested type.
    pub fn entity_type(&self) -> Option<EntityType> {
        match self {
            Self::Actor(actor) => Some(actor.entity_type()),
            Self::Url(_) => None,
            Self::BasicData(object) => Some(object.entity_type()),
        }
    }
}

/// Helper enumeration to wrap all sorts of Actor references in even more
/// weird scenarios.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum CompoundActorReference {
    /// Single actor reference, common case.
    Reference(ActorReference),
    /// Multiple actors references, somewhat rare case.
    /// For example content object could reference both actor and actor's group.
    List(Vec<ActorReference>),
}

impl CompoundActorReference {
    /// Returns ID of actor.
    /// In case of multiple actors returns ID of first Person-like entity in list or ID
    /// of first actor in list if there is no any Person-like entity.
    pub fn id(&self) -> Option<&url::Url> {
        match self {
            Self::Reference(actor_ref) => Some(actor_ref.id()),
            Self::List(actor_refs) => {
                // Some services like Peertube report multiple actors, e.g. Person and Group
                // As only single attribution entity stored per document in current index schema
                // there is need to choose which.

                // First try to find non-group actor
                let person_like_reference = actor_refs.iter()
                    .find(|x|
                        match x.entity_type() {
                            None => false,
                            Some(entity_type) => matches!(
                                entity_type,
                                EntityType::Person | EntityType::Service
                            )
                        }
                    );

                if let Some(actor_reference) = person_like_reference {
                    return Some(actor_reference.id());
                }

                // ok, any actor reference will do now
                actor_refs.iter()
                    .next()
                    .map(|v| v.id())
            }
        }
    }

    /// Returns single or multiple IDs as vector for a convenience of processing.
    pub fn as_id_vec(&self) -> Vec<&url::Url> {
        match self {
            CompoundActorReference::Reference(reference) => vec![reference.id()],
            CompoundActorReference::List(list) => list.iter()
                .map(|item| item.id())
                .collect()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Endpoints {
    /// Proxy endpoint to access objects that potentially require auth.
    #[cfg(feature = "more_properties")]
    #[serde(rename = "proxyUrl")]
    proxy_url: Option<url::Url>,

    /// For OAuth specifies where client could acquire authorisation tokens.
    #[cfg(feature = "more_properties")]
    #[serde(rename = "oauthAuthorizationEndpoint")]
    oauth_authorization_endpoint: Option<url::Url>,

    /// For OAuth specifies where client could acquire access tokens.
    #[cfg(feature = "more_properties")]
    #[serde(rename = "oauthTokenEndpoint")]
    oauth_token_endpoint: Option<url::Url>,

    /// Endpoint to authorise client public keys for client-to-server interactions.
    #[cfg(feature = "more_properties")]
    #[serde(rename = "provideClientKey")]
    provide_client_key: Option<url::Url>,

    /// Endpoint where client keys could be signed on behalf of actor.
    #[cfg(feature = "more_properties")]
    #[serde(rename = "signClientKey")]
    sign_client_key: Option<url::Url>,

    /// Endpoint for delivery of publicly addressed activities.
    /// Occasionally non-public addressed activities are delivered to it as well.
    #[serde(rename = "sharedInbox")]
    pub shared_inbox: Option<url::Url>,
}

/// This structure represents public key details.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PublicKey {
    /// ID of key
    pub id: url::Url,

    /// Who owns the key, actor itself is the usual owner.
    pub owner: url::Url,

    /// pem file condensed into single lines with end of lines escaped as `\n`.
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}

/// Helper enumeration to wrap public key.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum PublicKeyReference {
    /// Single key is present.
    Single(PublicKey),

    /// Multiple keys are present or maybe single key object is stored in JSON array.
    List(Vec<PublicKey>),
}

impl PublicKeyReference {
    /// This method returns nested public keys as a vector.
    pub fn as_vec(&self) -> Vec<&PublicKey> {
        match self {
            PublicKeyReference::Single(public_key) => vec![public_key],
            PublicKeyReference::List(public_keys) => public_keys.iter().collect()
        }
    }
}

/// Helper structure to represent actor as username and server it belongs to.
pub struct ActorReadableId {
    pub server: String,
    pub username: String,
}

/// This function returns discoverability state for `searchable_by` property.
/// Content is discoverable if `searchable_by` contains either well-known Public reference
/// or ot contains Fedineko specific not-really-used-by-anyone reference.
pub fn is_public_searchable_by(searchable_by: &[url::Url]) -> Option<Discoverable> {
    for url in searchable_by.iter() {
        match url.as_str() {
            "https://www.w3.org/ns/activitystreams#Public" |
            "https://fedineko.org/indexing#Public" => return Some(
                Discoverable::Allowed(AllowReason::SearchableBy(url.to_string()))
            ),
            _ => {}
        }
    }

    None
}
