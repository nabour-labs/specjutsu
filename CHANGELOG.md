# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[SemVer](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-11-13

### Added
- Cargo workspace skeleton (`sj-cli`, `sj-core`, `sj-runners`, `sj-adapters`).
- `sj trace`: map OpenSpec scenario IDs to Rust tests and report gaps.
- `sj gate --base <ref>`: CI gate with red-on-base (new tests must fail on the base
  code and pass on head), exemptions, `--json`, exit codes 0/1/2.
- `sj gate --since <commit>`: check every `feat(ID)` commit has an earlier `test(ID): red`.
- `cargo test` runner with libtest result parsing.

[Unreleased]: https://github.com/nabour-labs/specjutsu/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/nabour-labs/specjutsu/releases/tag/v0.2.0
