# Writing good tests

`sj test` and the ledger enforce the order: red, then green, on different code. They can't
tell whether the test is any good. A test that passes for the wrong reason proves nothing,
and the gate will still call the scenario proven. This file covers that judgment.

## A test proves one scenario

- **Assert the THEN, with its values.** If the scenario says "the response is 200 with a
  token valid for 24 hours", assert the 200 and the 24 hours. "Returns something" or "does
  not panic" is not the THEN.
- **One behaviour per test.** If the name needs "and", it is probably two scenarios. Split
  the test, or ask whether the scenario should be split.
- **Name it after the behaviour:** `fn auth_005_refreshes_a_token_expiring_in_5_minutes`,
  not `fn test_refresh_2`.
- **Use the scenario's concrete values**, not values you made up. If the scenario has none,
  that's a spec gap. Ask; don't invent them.

## FOR ALL scenarios need a property test

A scenario with a `- **FOR ALL**` step needs a property test that generates the inputs:
`proptest! { ... }` or `#[proptest]` in Rust, `@given(...)` from `hypothesis` in Python,
`fc.assert(fc.property(...))` or `fc.asyncProperty(...)` in TypeScript or JavaScript, with
`fc` imported from `fast-check`. Example tests may sit beside it: they don't have to fail on
base, but they must pass at head. Only property tests count as red-on-base evidence. Without
one, `sj gate` fails `not-property <ID> <test>`.

## Watch it fail for the right reason

Run `sj test <ID>` and read the failure excerpt before you write any code.

- **Right reason:** the assertion fails because the behaviour is missing ("expected 200, got
  404"), or it doesn't compile because the function doesn't exist yet.
- **Wrong reason:** a typo, a wrong import, a fixture that doesn't load, a panic in setup.
  Fix the test and run it again until it fails on the assertion that matters.
- **It passes straight away:** either the behaviour already exists or the test doesn't test
  it. Find out which before you go on. If the behaviour exists and must stay, it belongs
  in a `[keep]` scenario; tell the controller. Never leave a test you have only seen pass.

## Test behaviour, not mocks

- Call the code through its public interface: the function, the command, the endpoint.
- Mock only what you don't own or can't run in a test: the network, the clock, a paid API.
  Never mock the code under test.
- If you have to assert that a mock was called, check what the real result would be
  instead. A test that only checks the mock passes whatever the real code does.
- Don't add methods, flags or hooks to production code just so a test can reach in. If the
  code is hard to test, the design is telling you something; mention it in your report.

A test that only checks the mock, and one that checks the result:

```rust
// Passes whatever `refresh` returns: it only proves the clock was asked.
#[test]
fn auth_005_refreshes_token() {
    let clock = MockClock::new();
    let _ = refresh(&clock, &token());
    assert!(clock.was_called());
}

// Asserts the THEN: a new token, valid for 24 hours.
#[test]
fn auth_005_refreshes_a_token_expiring_in_5_minutes() {
    let clock = FixedClock::at("2026-01-01T12:00:00Z");
    let fresh = refresh(&clock, &token_expiring_in(Duration::from_secs(300))).unwrap();
    assert_eq!(fresh.expires_at, clock.now() + Duration::from_secs(24 * 3600));
}
```

## Make it pass with the least code

- Write only what the failing test needs. No options, settings or branches no scenario asks
  for. You aren't going to need them, and nobody has tested them.
- Hard-coding the expected value to get green is not done. If a second value from the
  scenario would break it, the code isn't finished.
- Run the package's tests (`sj test <package>`), not just the one, before you call it
  green.

## Refactor only while green

- Rename, extract and simplify only with every test passing, and run `sj test <ID>` after
  each step.
- A refactor that changes behaviour is not a refactor. Stop, and write the scenario first.

## Rationalizations

| Thought | Reality |
| --- | --- |
| "I'll write the test after; it's the same thing." | A test written after the code is shaped by the code, and you never saw it fail. |
| "The test is obvious, I don't need to watch it fail." | Obvious tests fail for the wrong reason most often. |
| "Mocking it is faster." | You are now testing the mock. |
| "I'll add the option now, we'll need it." | No scenario, no test, no option. |
| "It's too simple to test." | Then the test takes a minute. Write it. |
| "I'll just delete the test, it's flaky." | A flaky test is a bug. Use `sj-debug`. |

## Red flags

- You wrote implementation code before you saw `sj test <ID>` fail.
- The failure message is about setup, not about the behaviour.
- The test would still pass if you deleted the code you just wrote.
- The test asserts a mock was called, and nothing else.
- You changed an assertion to match what the code does instead of what the scenario says.
