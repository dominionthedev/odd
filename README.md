# odd

A weird way to deal with the filesystem.

Odd is a content-addressed object store for anything you `remember` — files
and, soon, directories — kept in a durable store outside your project, so
deleting the project never destroys what you captured. It is not a file
explorer, not a VFS, and not trying to replace Git.

## Status

Early. This is the first working vertical slice, not a finished tool:

```
odd init                          # bind a workspace to a store
odd remember <path> [--as <name>] # capture a file into the store
odd show <name|id>                # inspect a remembered object
odd resurrect <name|id>           # materialize it back to disk
```

Directories, composition (`graft`/`derive`/`splice`/`project`), the
operation log, and the watcher-driven commands (`haunt`/`mirror`/`shadow`)
are designed but not yet built — see the design notes (kept outside this
repo) for the full shape.

## Building

```
cargo build
cargo test
```

`scripts/check.sh` runs the same checks CI does: `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`.

## Design principles this codebase holds itself to

- **Structural identity only.** An object's id is derived from its content
  or structure — never from a path, an mtime, or any other filesystem fact.
- **Path-specific metadata lives on the binding, not the object.** Two
  names that dedupe to the same content each keep their own original path,
  mode, and mtime — deduping content must never mean losing one binding's
  facts to the other's.
- **Exact-length id resolution.** A reference is treated as an object id
  only when it is exactly the length of a content hash. Everything else
  resolves as a name. No length-threshold guessing.
- **Nothing ships ahead of its consumer.** No command, config field, or
  public function exists in this codebase without something that actually
  calls it. If you find one, that's a bug in the process, not just the code.

See `CONTRIBUTING.md` for how changes to this repo are expected to look.
