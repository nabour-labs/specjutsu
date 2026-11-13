# specjutsu

**Master the art of the spec.**

specjutsu is an open-source, single-binary CLI that makes AI coding agents prove
their work: every spec scenario gets a test, every test fails first, and CI
refuses anything unproven.

> Keep your spec tool. specjutsu proves the code matches it.

**Status:** pre-alpha (stage 0). `sj trace` works for OpenSpec specs and Rust
tests. See the [design doc](docs/specjutsu%20%E2%80%94%20Project%20Kickoff%20%26%20Design.md)
and [analysis](docs/analysis.md).

## Quickstart (from source)

```bash
cargo install --path crates/sj-cli
sj trace            # map scenario IDs in openspec/ to tests; list gaps
sj trace --json     # machine-readable
```

## How tracing works

Give every scenario a stable ID:

```markdown
#### Scenario: AUTH-003 Reject expired token
- **WHEN** the client calls GET /me with a token issued 25 hours ago
- **THEN** the response is 401 with code TOKEN_EXPIRED
```

Reference it from a test, by name or by comment:

```rust
#[test]
fn auth_003_rejects_expired_token() { /* ... */ }

// spec: AUTH-003
#[test]
fn rejects_expired() { /* ... */ }
```

`sj trace` reports which scenarios are covered, which have no test, and which
tests point at scenarios that no longer exist.

## Roadmap

| Release | Delivers |
| --- | --- |
| v0.1 | `sj trace` on OpenSpec + `cargo test`; non-blocking CI report |
| v0.2 | `sj gate` with red-on-base |
| v0.3 | `status`, `next`, `test`; Claude Code adapter; self-hosting |
| v0.5 | All runners and agents, importers, token benchmark, public launch |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Adding a test runner or an agent adapter
is the intended first PR.

## License

[MIT](LICENSE)
