## Why

Since `more-adapters-2` merged into main with its change still open, every PR fails the gate
with `green-on-base ADAPT-012`, even a branch identical to main. ADAPT-012 is in
`specs/adapters/spec.md` and, reworded, in the change's delta spec. When an ID appears twice,
the last occurrence wins. Head reads `specs/` before `changes/`, so the delta wins; base reads
paths in git tree order, where `changes/` sorts before `specs/`, so the old text wins. The
gate then sees a modification that no branch made.

## What Changes

- The gate reads base spec files in the same order as head: `specs/` first, then each open
  change in name order. A change's delta wins on both sides.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `gate`: a new requirement states which text wins when an ID appears in several files.

## Impact

- `sj-core::gate::prepare`: orders base spec paths before reading them.
