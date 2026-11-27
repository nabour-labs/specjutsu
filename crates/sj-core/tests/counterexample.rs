//! GATE scenarios from specjutsu/changes/property-counterexamples: reading the shrunk
//! counterexample out of a property test's failure text.

use proptest::prelude::*;
use sj_core::counterexample::counterexample;

#[test]
fn gate_069_hypothesis_falsifying_example_over_several_lines_becomes_one_line() {
    let text = "FAILED tests/test_pay.py::test_pay_020_refund_never_exceeds_charge\n\
                Falsifying example: test_pay_020_refund_never_exceeds_charge(\n    \
                charge=100,\n    \
                refund=101,\n\
                )\n";
    assert_eq!(
        counterexample(text).as_deref(),
        Some("charge=100, refund=101")
    );
}

#[test]
fn gate_069_hypothesis_brackets_inside_quoted_strings_do_not_end_the_example() {
    let text = "Falsifying example: test_pay_020_refund_never_exceeds_charge(\n    \
                note=')',\n    \
                memo=\"a\\\"(b\",\n\
                )\n";
    assert_eq!(
        counterexample(text).as_deref(),
        Some("note=')', memo=\"a\\\"(b\"")
    );
}

#[test]
fn gate_069_hypothesis_explain_comments_are_left_out() {
    let text = "Falsifying example: test_pay_020_refund_never_exceeds_charge(\n    \
                charge=0,  # or any other generated value (see below)\n    \
                note='#1',  # a ')' here doesn't close it\n    \
                refund=1,\n\
                )\n";
    assert_eq!(
        counterexample(text).as_deref(),
        Some("charge=0, note='#1', refund=1")
    );
}

#[test]
fn gate_073_a_cut_that_lands_on_a_space_leaves_no_space_before_the_ellipsis() {
    let text = format!(
        "minimal failing input: {} {}\n",
        "x".repeat(199),
        "y".repeat(50)
    );
    let expected = format!("{}…", "x".repeat(199));
    assert_eq!(counterexample(&text), Some(expected));
}

#[test]
fn gate_069_hypothesis_example_is_found_after_an_empty_proptest_marker() {
    let text = "minimal failing input:\n\
                Falsifying example: test_pay_020_refund_never_exceeds_charge(charge=100, refund=101)\n";
    assert_eq!(
        counterexample(text).as_deref(),
        Some("charge=100, refund=101")
    );
}

#[test]
fn gate_070_fast_check_counterexample_is_shown() {
    let text = "FAIL  PAY-020 refund never exceeds charge\n\
                Error: Property failed after 4 tests\n\
                { seed: 42, path: \"3:0\", endOnFailure: true }\n\
                Counterexample: [100,101]\n\
                Shrunk 2 time(s)\n";
    assert_eq!(counterexample(text).as_deref(), Some("[100,101]"));
}

#[test]
fn gate_073_long_counterexample_is_cut_to_200_characters() {
    let body = "x".repeat(300);
    let text = format!("thread panicked\nminimal failing input: items = [{body}]\n");
    let full = format!("items = [{body}]");
    let expected: String = full.chars().take(200).chain(['…']).collect();
    assert_eq!(counterexample(&text), Some(expected));
}

/// Failure text that often carries one of the three patterns, so every path of the parser
/// sees arbitrary input after its marker.
fn failure_text() -> impl Strategy<Value = String> {
    let marker = prop_oneof![
        Just(""),
        Just("minimal failing input:"),
        Just("Falsifying example: test_x("),
        Just("Counterexample:"),
    ];
    (
        "(?s).{0,1000}",
        marker,
        "(?s).{0,4000}",
        "[ \t\r\n(),=xé€]{0,4000}",
    )
        .prop_map(|(head, marker, a, b)| {
            format!("{head}{marker}{a}{b}")
                .chars()
                .take(10_000)
                .collect::<String>()
        })
}

proptest! {
    #[test]
    fn gate_075_reading_a_counterexample_never_fails_and_returns_one_short_line(
        text in failure_text()
    ) {
        if let Some(line) = counterexample(&text) {
            assert!(
                !line.contains(['\n', '\r', '\u{2028}', '\u{2029}', '\u{85}']),
                "line break in {line:?}"
            );
            let n = line.chars().count();
            assert!(
                n <= 200 || (n == 201 && line.ends_with('…')),
                "{n} characters: {line:?}"
            );
        }
    }
}
