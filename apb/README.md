# apb
> traits and types for implementing [ActivityPub](https://www.w3.org/TR/activitypub/)

`apb` implements all [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) types as traits, so that implementing structs don't need to hold all possible fields, but only implement getters for relevant ones

`apb` also provides a `Node` enum, which can be Empty, Link, Object or Array

if the `unstructured` feature is enabled, all traits are implemented for `serde_json::Value`, so that it's possible to manipulate free-form json maps as valid AP objects

if the `orm` feature is enabled, enum types are also database-friendly with sea-orm

if the `fetch` feature is enabled (together with `unstructured`), `Node`s expose a `async fn fetch(&mut self) -> reqwest::Result<&mut Self>` to dereference remote nodes

## why
[upub](https://git.alemi.dev/upub.git) uses these types to implement its federation, but I wanted to modularize apb types. this crate is still work in progress and favors upub's needs, get in touch if you'd like to help or tune apb to your project!
