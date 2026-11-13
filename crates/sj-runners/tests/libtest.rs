//! GATE scenarios covered by the cargo runner (openspec/changes/gate-red-on-base).

use sj_runners::cargo::{parse_libtest, Outcome, Run, TestResult};

const OUTPUT: &str = "running 4 tests\ntest tests::adds ... ok\ntest subtracts ... FAILED\ntest slow ... ignored\ntest panics - should panic ... ok\n\ntest result: FAILED. 2 passed; 1 failed; 1 ignored\n";

#[test]
fn gate_005_parses_libtest_results() {
    let got = parse_libtest(OUTPUT);
    let expect = vec![
        ("tests::adds", Outcome::Passed),
        ("subtracts", Outcome::Failed),
        ("slow", Outcome::Ignored),
        ("panics", Outcome::Passed),
    ];
    let got: Vec<(&str, Outcome)> = got.iter().map(|r| (r.path.as_str(), r.outcome)).collect();
    assert_eq!(got, expect);
}

#[test]
fn gate_006_outcome_matches_by_function_name() {
    let run = Run {
        success: false,
        results: vec![
            TestResult {
                path: "tests::adds".into(),
                outcome: Outcome::Passed,
            },
            TestResult {
                path: "subtracts".into(),
                outcome: Outcome::Failed,
            },
        ],
    };
    assert_eq!(run.outcome("adds"), Some(Outcome::Passed));
    assert_eq!(run.outcome("subtracts"), Some(Outcome::Failed));
    assert_eq!(run.outcome("dds"), None);
    assert_eq!(run.outcome("missing"), None);
}
