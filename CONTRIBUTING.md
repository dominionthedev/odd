# Contributing

This is a solo project, but the rules below exist so the codebase doesn't
drift the way an earlier attempt at this project did — every rule here maps
to a specific failure that's already been seen and isn't worth repeating.

## Commits

- Conventional commits: `type(scope): message` — `feat`, `fix`, `chore`,
  `docs`, `test`.
- One logical change per commit. No bundling unrelated changes.
- **Every commit must build and pass `cargo test` on its own.** Not "the
  branch builds at the end" — each individual commit. If a change can't be
  made that way, it's not one change yet; split it further.

## Scope discipline

- No command, public function, config field, or relation type gets added
  without something in the same commit that actually uses it. Untested,
  unwired scaffolding is exactly how the old codebase accumulated dead
  `pub fn`s and config fields nobody read.
- A bugfix session does not grow into a new-feature session mid-stream.
  State which one a change is before writing it.

## Testing

- A test must actually fail without the fix or feature it covers.
  Coverage for its own sake isn't the goal.
- Structural/content-identity logic gets unit tests before any command
  is allowed to call it — not after.

## Naming

- Non-descriptive, thematic names are fine and preferred over literal
  ones (`graft`, `molt`, `fossil` over `merge`, `version`, `tag`). Rename
  only when a name has become actively misleading, not just imperfect.

## Design docs

- Architectural questions get resolved in discussion before implementation
  starts, not worked out inside a PR.
- The full command/operation design lives outside this repo. If you're
  reading this without that context, ask before adding a new top-level
  command — the shape of the CLI surface is deliberate, not incremental
  guesswork.
