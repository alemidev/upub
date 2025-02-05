# upub-web

![screenshot of upub frontend](https://cdn.alemi.dev/proj/upub/fe/20240704.png)

this is μpub main frontend: it's a single wasm bundle doing client-side routing for faster navigation between objects

it has the drawback of not being search-engine friendly, but machines should process data for machines themselves (aka: the AP documents), so it's probably fine to have a "js"-heavy frontend

## development

it's probably possible to get `upub-web` to build with just `wasm-bindgen`, but i recommend just using `trunk` to keep your sanity. trunk will download by itself `wasm-bindgen-cli`.

```
UPUB_BASE_URL=https://dev.upub.social trunk serve --public-url http://127.0.0.1:8080/web
```

will give you a local development server with auto-reload pointing to `dev.upub.social`, so you don't even need to spin up a local instance (omit `UPUB_BASE_URL` env variable to make frontend point to localhost instead)

setting `public-url` is necessary: while deploying, axum will handle getting you to the correct pages, but in trunk dev env this has to be done by trunk itself. note also that resource urls will differ: on prod they will be `/web/assets/style.css`, while on dev `/assets` segment won't be present!

## building

just run

```
$ trunk build --release
```

to generate the `./web/dist` folder containing all necessary assets

either serve these yourself, or compile main `upub` with `web` feature enable to have it bundle these freshly built frontend files
