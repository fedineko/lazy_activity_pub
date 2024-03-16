# What is this?

It is quite a messy implementation of a few data entities defined in ActivityPub
spec.

* Test coverage is poor, fine to say it does not really exist.
* Data entities are just a few.
* Object properties for deserialization are chosen on a whim, just for a random
  needs of Fedineko project and to support most of the payloads in incoming
  activities.
* Documentation... make your guess.
* Security audit is not a thing.
* and more!

On the other hand `lazy_activitypub` does not have too many dependencies.

# Usage
This library essentially is a set of almost POJO data entities for deserialization,
so it is not that useful by itself, there is need to get those serialized entities
somewhere and then use `serde_json` to deserialize:
```rust
    let serialized: &str = get_string_to_process(); 
    let activity: Activity = serde_json::from_str(serialized);
```

# License
Apache 2.0 or MIT.