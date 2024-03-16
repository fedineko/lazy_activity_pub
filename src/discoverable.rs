/// Reason why content or actor is allowed to index.
#[derive(Debug)]
pub enum AllowReason {
    /// `discoverable` flag is set to `true`
    Discoverable,
    /// `indexable` flag is set to `true`
    Indexable,
    /// `fedineko:index` property is set to `allow`
    FedinekoProperty,
    /// `searchableBy` scope contains well-known public address
    /// or Fedineko address.
    SearchableBy(String),
    /// If there is no explicit deny to index on actor level,
    /// then assumption is that indexing is allowed.
    Assumed,
}

#[derive(Debug)]
pub enum DenyReason {
    /// `discoverable` flag is set to `false`
    Discoverable,
    /// `indexable` flag is set to `false`
    Indexable,
    /// `fedineko:index` property is set to any value other than `allow`
    FedinekoProperty,
    /// There is explicit opt-out request.
    OptedOut,
    /// Account is banned.
    Ban,
    /// When content level permissions are checked,
    /// default is to deny indexing unless there is explicit opt-in.
    Default,
}

/// This enumeration indicates whether indexing is allowed for actor
/// or content.
#[derive(Debug)]
pub enum Discoverable {
    /// Yes, could do some indexing.
    Allowed(AllowReason),
    /// No, no indexing or storing data.
    Denied(DenyReason),
}

impl Discoverable {
    /// This helper method returns `true` if indexing is allowed by any
    /// [AllowReason].
    pub fn is_allowed_indexing(&self) -> bool {
        matches!(self, Self::Allowed(_))
    }
}
