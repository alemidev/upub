<p align="center">
  <img src="https://dev.upub.social/web/assets/icon.png" alt="upub logo: greep mu letter with blue and pink-reddish gradient" height="150" />
</p>

# μpub

> ## [micro social network, federated](https://join.upub.social)
>
> - [about](#about)
>   - [features](#features)
>   - [security](#security)
>   - [caching](#caching)
> - [deploy](#deploy)
>   - [install](#install)
>   - [run](#run)
>   - [configure](#configure)
> - [development](#development)
>   - [contacts](#contacts)
>   - [contributing](#contributing)

# about
μpub aims to be a private, lightweight, modular and **secure** [ActivityPub](https://www.w3.org/TR/activitypub/) server

μpub is modeled around timelines but tries to be unopinionated in its implementation, allowing representing multiple different fediverse "modalities" together

all client interactions happen with ActivityPub's client-server methods (basically POST your activities to your outbox), with [appropriate extensions](https://ns.alemi.dev/as): **μpub doesn't want to invent another API**!

> [!NOTE]
> a test instance is available at [dev.upub.social](https://dev.upub.social)

## features
μpub boasts both known features and new experimental ideas:
 * quote posts, groups, tree view
 * media proxy: minimal local storage impact
 * AP explorer: navigate underlying AP documents
 * on-demand thread fetching: get missing remote replies on-demand
 * granular activity privacy: control who gets to see each of your likes and shares
 * actor liked feeds: browse all publicly likes content from users, as "curated timelines"

## security
most activitypub implementations don't really validate fetches: knowing an activity/object id will allow anyone to resolve it on most fedi software. this is of course unacceptable: "security through obscurity" just doesn't work

μpub correctly and rigorously implements and enforces access control on each object based on its addressing

> [!IMPORTANT]
> most instances will have "authorized fetch" which kind of makes the issue less bad, but anyone can host an actor, have any server download their pubkey and then start fetching

μpub may be considered to have "authorized fetch" permanently on, except it depends on each post:
 * all incoming activities must be signed or will be rejected
 * all posts marked public (meaning, addressed to `https://www.w3.org/ns/activitystreams#Public`), will be fetchable without any authorization
 * all posts not public will require explicit addressing and authentication: for example if post A is addressed to example.net/actor
   * anonymous fetchers will receive 404 on GET /posts/A
   * local users must authenticate and will be given said post only if it's addressed to them
   * remote servers will be given access to all posts from any of their users once they have authenticated themselves (with http signing)

> [!TIP]
> note that followers get expanded at insertion time: addressing to `example.net/actor/followers` will address to anyone following actor that the server knows of, **at that time**

## caching
μpub **doesn't download remote media** to both minimize local resources requirement and avoid storing media that remotes want gone. to prevent leaking local user ip addresses, all media links are cloaked and proxied.

while this just works for small instances, larger servers should set up aggressive caching on `/proxy/...` path: more info [in following sections](#media-proxy-cache)

# deploy
μpub is built with the needs of small deployments in mind: getting a dev instance up is as easy as running one command, and setting up for production just requires some config tweaking

## install
latest μpub build can be downloaded from [moonlit.technology releases page](https://moonlit.technology/alemi/upub/releases)

```sh
curl -s https://moonlit.technology/alemi/upub/releases/download/v0.5.0/upub > ~/.local/bin/upub; chmod +x ~/.local/bin/upub
```
> [!IMPORTANT]
> automated cross-platform builds by GitHub are planned and will be made available soon

### from source
building μpub from source is also possible without too much effort. it will also allow to customize the resulting binary to your specific use case

if you just want to build the backend (or some of its components), a simple `$ cargo build` will do

---

to also build upub-web, some extra tooling must be installed:
 * rust `wasm32-unknown-unknown` target (`$ rustup target add wasm32-unknown-unknown`)
 * wasm-bindgen (`$ cargo install wasm-bindgen-cli`)
 * trunk (`$ cargo install trunk`)
 
from inside `web` project directory, run `trunk build --release`. once it finishes, a `dist` directory should appear inside `web` project. it is now possible to build μpub with the `web` feature flag enabled, which will include upub-web frontend

```sh
cd web
trunk build --release
cd ..
cargo build --release --features=web
```

## run
μpub includes its maintenance tooling and different operation modes, all documented in its extensive command line interface.

> [!TIP]
> make sure to use `--help` if you're lost! each subcommands has its own help screen

all modes share `-c`, `--db` and `--domain` options, which will set respectively config path, database connection string and instance domain url.
none of these is necessary: by default a sqlite database `upub.db` will be created in current directory, default config will be used and domain will be a localhost http url

bring up a complete instance with `monolith` mode: `$ upub monolith`: it will:
 * run migrations
 * setup frontend and routes
 * spawn a background worker

most maintenance tasks can be done with `$ upub cli`: register a test user with `$ upub cli register user password`

done! try connecting to http://127.0.0.1:3000/web

## configure
all configuration lives under a `.toml` file. there is no default config path: point to it explicitly with `-c` flag while starting `upub`

> [!TIP]
> to view μpub full default config, use `$ upub config`

a super minimal config may look like this

```toml
[instance]
name = "my-upub-instance"
domain = "https://my.domain.social"

[datasource]
connection_string = "postgres://localhost/upub"

[security]
allow_registration = true
```

### moderation
> [!CAUTION]
> currently there aren't many moderation tools and tasks will require querying db directly

interactions with remote instances can be finetuned using the `[reject]` table:

```
# discard incoming activities from these instances
incoming = []

# prevent fetching content from these instances
fetch = []

# prevent content from these instances from being displayed publicly
# this effectively removes the public (aka NULL) addressing = only other addressees (followers,
# mentions) will be able to see content from these instances on timelines and directly
public = []

# prevent proxying media coming from these instances
media = []

# skip delivering to these instances
delivery = []

# prevent fetching private content from these instances
access = []

# reject any request from these instances (ineffective as they can still fetch anonymously)
requests = []
```

### media proxy cache
caching proxied media is quite important for performance, as it keeps proxying load away from μpub itself

for example, caching `nginx` could be achieved this way:
```nginx
proxy_cache_path /tmp/upub/cache levels=1:2 keys_zone=upub_cache:100m max_size=50g inactive=168h use_temp_path=off;

server {
	location /proxy/ {
		# use our configured cache
		slice              1m;
		proxy_set_header   Range $slice_range;
		chunked_transfer_encoding on;
		proxy_ignore_client_abort on;
		proxy_buffering    on;
		proxy_cache        upub_cache;
		proxy_cache_key    $host$uri$is_args$args$slice_range;
		proxy_cache_valid  200 206 301 304 168h;
		proxy_cache_lock   on;
		proxy_pass         http://127.0.0.1/;
	}
}

```

### polylith
it's also possible to deploy μpub as multiple smaller services, but this process will require some expertise as the setup is experimental and poorly documented

multiple specific binaries can be compiled with various feature flags:
 - `cli` and `migrations` are only required at maintenance time
 - `worker` processes jobs, only interacts with database
 - `serve` answers http requests, can also queue jobs POSTed on inboxes, can be further split into
   - `activitypub` with core AP routes
   - `web` serving static frontend

remember to prepare config file and run migrations!

# development
development is still active, so expect more stuff to come! since most fediverse software uses Mastodon's API, μpub plans to implement it as an optional feature, becoming eventually compatible with most existing frontends and mobile applications, but focus right now is on producing something specific to μpub needs

## contacts
 * new features or releases are announced [directly on the fediverse](https://dev.upub.social/actors/upub)
 * direct questions about deployment or development, or general chatter around this project, [happens on matrix](https://matrix.to/#/#upub:moonlit.technology)
 * development mainly happens on [moonlit.technology](https://moonlit.technology/alemi/upub), but a [github mirror](https://github.com/alemidev/upub) is also available. if you prefer a forge-less development you can browse the repo on [my cgit](https://git.alemi.dev/upub.git), and send me patches on any contact listed on [my site](https://alemi.dev/about/contacts)

## contributing
μpub can always use more dev time!

if you want to contribute you will need to be somewhat familiar with [rust](https://www.rust-lang.org/): even the frontend is built with it!

reading a bit of the [ActivityPub](https://www.w3.org/TR/activitypub/) specification can be useful but not really required

hanging out in the relevant matrix room will probably be useful, as you can ask questions while familiarizing with the codebase

once you feel ready to tackle some development, head over to [the issues tab](https://moonlit.technology/alemi/upub/issues) and look around for something that needs to be done!

