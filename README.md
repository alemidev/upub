# μpub
> micro social network, federated

μpub aims to be a private, lightweight, modular and **secure** [ActivityPub](https://www.w3.org/TR/activitypub/) server

μpub is currently being developed and can do most basic things, like posting notes, liking things, following others, deliveries and browsing

all interactions must happen with ActivityPub's client-server methods (basically POST your activities to your outbox), and there's a simple frontend

a test instance is _usually_ available at [feditest.alemi.dev](https://feditest.alemi.dev)

upub's stock frontend is also being developed and can be viewed _usually_ at [feditest.alemi.dev/web](https://feditest.alemi.dev/web)

## about security
most activitypub implementations don't really validate fetches: knowing an activity/object id will allow anyone to resolve it on most fedi software. this is of course unacceptable: "security through obscurity" just doesn't work

μpub correctly and rigorously implements and enforces access control on each object based on its addressing

most instances will have "authorized fetch" which kind of makes the issue less bad, but anyone can host an actor, have any server download their pubkey and then start fetching

μpub may be considered to have "authorized fetch" permanently on, except it depends on each post:
 * all posts marked public (meaning, addressed to "https://www.w3.org/ns/activitystreams#Public"), will be fetchable without any authorization
 * all posts not public will require explicit addressing and authentication: for example if post A is addressed to example.net/actor
   * anonymous fetchers will receive 404 on GET /posts/A
   * local users must authenticate and will be given said post only if it's addressed to them
   * remote servers will be given access to all posts from any of their users once they have authenticated themselves (with http signing)

note that followers get expanded: addressing to example.net/actor/followers will address to anyone following actor that the server knows of, at that time

## progress

 - [x] barebone actors
 - [x] barebone activities and objects
 - [x] activitystreams/activitypub compliance (well mostly)
 - [x] process barebones feeds
 - [x] process barebones inbox
 - [x] process barebones outbox
 - [x] http signatures
 - [x] privacy, targets, scopes
 - [x] simple web client
 - [ ] announce (boosts)
 - [ ] threads
 - [ ] editing
 - [ ] searching
 - [ ] media
 - [ ] user fields
 - [ ] mastodon api
 - [ ] hashtags, discovery
 - [ ] polls
 - [ ] lists
 - [ ] more optimized database schema

## what about the name?
μpub, sometimes stylyzed `upub`, is pronounced `mu-pub` (the `μ` stands for [micro](https://en.wikipedia.org/wiki/International_System_of_Units#Prefixes))

## frontend
upub aims to be compatible with multiple frontends via the mastodon api, but a simple custom ui is also being worked on

![screenshot of upub simple frontend](https://cdn.alemi.dev/proj/upub/fe.png)
