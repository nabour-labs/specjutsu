## Context

`gate::classify` keeps the last occurrence of an ID on each side ("a change's delta overrides
the current spec"). Head files come from `spec::discover_spec_files` (`specs/`, then
`changes/<name>/specs/` sorted by change name); base files come from `git ls-tree` order.

## Goals / Non-Goals

**Goals:** the same tree on base and head gives every scenario `unchanged`.

**Non-Goals:** changing which text wins. The delta still overrides `specs/`.

## Decisions

- **Sort base paths into head's order** (`specs/` before `changes/`, then by path) in
  `prepare`. Rejected: making `classify` order-independent by merging per file, which needs
  file paths in `Fingerprint` and touches every caller for the same result.

## Risks / Trade-offs

- [Two files inside `specs/` with the same ID sort differently in git and on disk] → both
  sides sort by path bytes after the `specs/`-first key, so the order matches.
