use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use url::Url;
use crate::actor::ActorReadableId;

/// Helper function to build case insensitive regex from
/// given [pattern].
fn build_regex(pattern: &str) -> Regex {
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
        .unwrap()
}

/// List of known actor URLs formats.
/// `user` capture group is used by functions in this module
/// to extract account name from URL.
///
/// While there is no guarantee this "username" matches actual
/// username, in practice it is quite often matches.
static ACTOR_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        build_regex(r"^/(users|u)/(?P<user>[^/]+)$"),
        build_regex(r"^/profile/(?P<user>[^/]+)$"),
        build_regex(r"^/ap/users/(?P<user>\d+)$"),
    ]
});

/// List of known content URLs formats.
static CONTENT_REGEXES: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        build_regex(r"^/users/[^/]+/statuses/\d+$"),    // Mastodon-like
        build_regex(r"^/notes/.+$"),                    // Misskey-like
        build_regex(r"^/p/([^/]+)/\d+$"),               // Pixelfed-like
        build_regex(r"^/post/\d+$"),                    // Lemmy-like
        build_regex(r"^/(notice|objects)/[^/]+$"),      // Soapbox-like
        build_regex(r"^/ap/users/\d+/post/\d+/?$"),     // threads.net scheme
    ]
});

/// This function matches [url] against known and extracts account
/// name from it if possible.
pub fn extract_username_from_url(url: &Url) -> Option<String> {
    let path = url.path();

    if path.is_empty() {
        return None;
    }

    for regex in ACTOR_REGEXES.iter() {
        if let Some(captures) = regex.captures(path) {
            let group_value = captures.name("user");

            if group_value.is_none() {
                continue;
            }

            return Some(group_value.unwrap().as_str().to_string());
        }
    }

    None
}

/// This function is similar to [extract_username_from_url()] but returns
/// [ActorReadableId] instead of just string. [ActorReadableId] includes
/// host name extracted from given [url] as well.
pub fn extract_actor_readable_id_from_url(url: &Url) -> Option<ActorReadableId> {
    if let Some(username) = extract_username_from_url(url) {
        if let Some(host) = url.host() {
            return Some(
                ActorReadableId {
                    server: host.to_string(),
                    username
                }
            );
        }
    }

    None
}


/// Enumeration used as return value for guessing functions.
#[derive(Debug, PartialEq)]
pub enum GuessedType {
    /// URL is assumed to be a link to Actor.
    ACTOR,

    /// URL is assumed to be a link to Content.
    CONTENT,

    /// It is not possible to guess what URL possibly refers to.
    UNKNOWN,
}


///  This method guesses type of referenced object from [url].
///  It is unreliable but if there is only object URL and not object definition itself or
///  if object is converted to tombstone this could serve as a last chance option.
///  Returns inferred [GuessedType].
pub fn guess_object_type_from_url(url: &Url) -> GuessedType {
    let path = url.path();

    if path.is_empty() {
        return GuessedType::UNKNOWN;
    }

    for regex in ACTOR_REGEXES.iter() {
        if regex.is_match(path) {
            return GuessedType::ACTOR;
        }
    }

    for regex in CONTENT_REGEXES.iter() {
        if regex.is_match(path) {
            return GuessedType::CONTENT;
        }
    }

    GuessedType::UNKNOWN
}


