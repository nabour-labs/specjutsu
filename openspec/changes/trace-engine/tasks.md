## 1. Spec parsing (sj-core)

- [x] 1.1 Parse `#### Scenario:` headings into IDs, names, lines; skip fenced code (TRACE-001, TRACE-002)
- [x] 1.2 Discover spec files under the spec root, excluding the archive (TRACE-003)

## 2. Test matching (sj-core)

- [x] 2.1 Detect Rust test fns and match IDs in names and `// spec:` comments (TRACE-004)

## 3. Report (sj-core + sj-cli)

- [x] 3.1 Build the trace report: mapped, gaps, orphans (TRACE-005)
- [x] 3.2 `sj trace` command with compact output, `--json`, `--strict`, `--root` (TRACE-006)
