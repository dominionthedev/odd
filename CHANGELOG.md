# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- `init` — bind a workspace directory to a store, defaulting the store's
  name to a slug of the containing directory instead of a raw UUID.
- `remember` — capture a single file into the store as a content-addressed
  blob, bound to a name in a namespace. Path-specific metadata (original
  path, mode, mtime) is recorded on the binding, not the object.
- `show` — inspect a remembered object by name or by exact object id.
- `resurrect` — materialize a remembered object back to disk, defaulting
  the destination to the binding's own original path.
- Content-addressed `Object` model (`File`, `Directory` kinds) with
  structural-id tests covering dedup, non-collision, and directory
  entry-order independence.
- CI: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` on every push.
