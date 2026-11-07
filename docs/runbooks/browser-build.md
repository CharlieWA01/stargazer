# The browser build (stargazer.wasm)

The portfolio's play page runs Stargazer in the visitor's browser from
`stargazer.wasm`, built from `web/` (DESIGN.md, "Runs in a browser").

## Build and check it locally

```sh
cargo test -p stargazer-web
cargo build -p stargazer-web --release --target wasm32-unknown-unknown
node web/tests/smoke.mjs target/wasm32-unknown-unknown/release/stargazer_web.wasm
```

The last line prints `ok: Stargazer 0.2.0, replied ... in ... ms`. CI runs
the same three on every PR.

## Ship a new version to the play page

1. Release as usual: bump `Cargo.toml` and merge, then tag the merged
   commit. Fetch first: `git tag` uses your copy of `origin/main`, which
   may be out of date.
   ```sh
   git fetch origin
   git log -1 --oneline origin/main     # the merge you expect?
   git tag vX.Y.Z origin/main && git push origin vX.Y.Z
   ```
   The `Release` workflow attaches `stargazer.wasm` and its SHA-256. (A
   tag on the wrong commit is refused by its version check; delete it with
   `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z` and tag again.)
2. In the portfolio repo: `tools/update-stargazer.sh vX.Y.Z` downloads it,
   checks the checksum and records the version; open a PR with the
   result. Merging deploys it.

The Lichess bot and the play page move independently: each names its own
release.
