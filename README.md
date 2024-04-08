# μpub
> micro social network, federated

μpub aims to be a fast, lightweight and secure [ActivityPub](https://www.w3.org/TR/activitypub/) server

μpub is currently being developed and can do some basic things, like posting notes, follows and likes

all interactions must happen with ActivityPub's client-server methods (basically POST your activities to your outbox)

a test instance is _usually_ available at [feditest.alemi.dev](https://feditest.alemi.dev)

## progress

 - [x] barebone actors
 - [x] barebone activities and objects
 - [x] activitystreams/activitypub compliance (well mostly)
 - [x] process barebones feeds
 - [x] process barebones inbox
 - [x] process barebones outbox
 - [x] http signatures
 - [ ] privacy, targets, scopes
 - [ ] more optimized database schema
 - [ ] hashtags, discovery
 - [ ] client api (mastodon/pleroma)
 - [ ] full activitystreams/activitypub support
 - [ ] a custom frontend maybe?

## what about the name?
μpub, sometimes stylyzed `upub`, is pronounced `mu-pub` (the `μ` stands for [micro](https://en.wikipedia.org/wiki/International_System_of_Units#Prefixes))
