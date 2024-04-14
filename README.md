# μpub
> micro social network, federated

μpub aims to be a fast, lightweight and secure [ActivityPub](https://www.w3.org/TR/activitypub/) server

μpub is currently being developed and can do most basic things, like posting notes, liking things, following others, deliveries and browsing

all interactions must happen with ActivityPub's client-server methods (basically POST your activities to your outbox), and there's a simple frontend

a test instance is _usually_ available at [feditest.alemi.dev](https://feditest.alemi.dev)

upub's stock frontend is also being developed and can be viewed _usually_ at [feditest.alemi.dev/web](https://feditest.alemi.dev/web)

## progress

 - [x] barebone actors
 - [x] barebone activities and objects
 - [x] activitystreams/activitypub compliance (well mostly)
 - [x] process barebones feeds
 - [x] process barebones inbox
 - [x] process barebones outbox
 - [x] http signatures
 - [x] privacy, targets, scopes
 - [ ] simple web client
 - [ ] announce (boosts)
 - [ ] threads
 - [ ] editing
 - [ ] searching
 - [ ] mastodon api
 - [ ] hashtags, discovery
 - [ ] more optimized database schema

## what about the name?
μpub, sometimes stylyzed `upub`, is pronounced `mu-pub` (the `μ` stands for [micro](https://en.wikipedia.org/wiki/International_System_of_Units#Prefixes))

## frontend
upub aims to be compatible with multiple frontends via the mastodon api, but a simple custom ui is also being worked on

![screenshot of upub simple frontend](https://cdn.alemi.dev/proj/upub/fe.png)
