# μpub
> micro social network, federated

![screenshot of upub simple frontend](https://cdn.alemi.dev/proj/upub/fe/20240514.png)

μpub aims to be a private, lightweight, modular and **secure** [ActivityPub](https://www.w3.org/TR/activitypub/) server

 * follow development [in the dedicated matrix room](https://matrix.to/#/#upub:alemi.dev)

μpub is usable as a very simple ActivityPub project: it has a home and server timeline, it allows to browse threads, star notes and leave replies, it renders remote media of any kind and can be used to browse and follow remote users

all interactions happen with ActivityPub's client-server methods (basically POST your activities to your outbox), with [appropriate extensions](https://ns.alemi.dev/as): **μpub doesn't want to invent another API**!

development is still active, so expect more stuff to come! since most fediverse software uses Mastodon's API, μpub plans to implement it as an optional feature, becoming eventually compatible with most existing frontends and mobile applications, but focus right now is on producing something specific to μpub needs

a test instance is _usually_ available at [feditest.alemi.dev](https://feditest.alemi.dev)

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

## contributing

all help is extremely welcome! development mostly happens on [moonlit.technology](https://moonlit.technology/alemi/upub.git), but there's a [github mirror](https://github.com/alemidev/upub) available too

if you prefer a forge-less development you can browse the repo on [my cgit](https://git.alemi.dev/upub.git), and send me patches on any contact listed on [my site](https://alemi.dev/about/contact)

don't hesitate to get in touch, i'd be thrilled to showcase the project to you!

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
 - [x] announce (boosts)
 - [x] threads
 - [x] remote media
 - [x] editing via api
 - [x] advanced composer
 - [x] api for fetching
 - [x] like, share, reply via frontend
 - [x] backend config
 - [x] frontend config
 - [ ] mentions, notifications
 - [ ] mastodon-like search bar
 - [ ] polls
 - [ ] better editing via web frontend
 - [ ] remote media proxy
 - [ ] upload media
 - [ ] hashtags
 - [ ] public vs unlisted for discovery
 - [ ] user fields
 - [ ] lists
 - [ ] full mastodon api
 - [ ] optimize `addressing` database schema

## what about the name?
μpub (or simply `upub`) means "[micro](https://en.wikipedia.org/wiki/International_System_of_Units#Prefixes)-pub", but could also be read "upub", "you-pub" or "mu-pub"
