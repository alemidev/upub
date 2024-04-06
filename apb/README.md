# apb
> traits and types for implementing [ActivityPub](https://www.w3.org/TR/activitypub/)

`apb` implements all [ActivityStreams](https://www.w3.org/TR/activitystreams-core/) types as traits, so that implementing structs don't need to hold all possible fields, but only implement getters for relevant ones

`apb` also provides a `Node<T>` enum, which can represent ActivityPub nodes (empty, link, object, array)

read more in this crate's docs


## why
[upub](https://git.alemi.dev/upub.git) uses these types to implement its federation, but I wanted to modularize apb types

## state
this crate is still work in progress and favors upub's needs, get in touch if you'd like to help or tune apb to your project!
