# upub-web

this is μpub main frontend: it's a single wasm bundle doing clientside routing for faster navigation between objects

it has the drawback of not being search-engine friendly, but machines should process data for machines themselves (aka: the AP documents), so it's probably fine to have a "js"-heavy frontend

## development

it's probably possible to get `upub-web` to build with just `wasm-bindgen`, but i recommend just using `trunk` to keep your sanity. by default `upub-web` sets the "offline" option, so you'll still need to download `wasm-bindgen` yourself (or run with `TRUNK_OFFLINE=false` and let trunk download it itself once)

```
$ UPUB_BASE_URL=https://dev.upub.social trunk serve
```

will give you a local development server with auto-reload pointing to `dev.upub.social`, so you don't even need to spin up a local instance (omit `UPUB_BASE_URL` env variable to make frontend point to localhost instead)

## building

just run

```
$ trunk build --release
```

to generate the `./web/dist` folder containing all necessary assets

either serve these yourself, or compile main `upub` with `web` feature enable to have it bundle these freshly built frontend files
