// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Tests for ordered state-transition result extraction.

use crate::extract;
use std::cell::RefCell;
use std::io;
use std::sync::Arc;
use test_with_tracing::test;

#[test]
fn extraction_filters_successes_in_input_order() {
    let input: Vec<(Arc<str>, Result<u64, io::Error>)> = vec![
        (Arc::from("first"), Ok(8)),
        (Arc::from("omitted"), Ok(5)),
        (Arc::from("last"), Ok(6)),
    ];
    let values = extract("save", input.into_iter(), |name, value| {
        (value % 2 == 0).then_some((name, value))
    })
    .unwrap();
    assert_eq!(values, [(Arc::from("first"), 8), (Arc::from("last"), 6)]);

    let empty: Vec<(Arc<str>, Result<u64, io::Error>)> = Vec::new();
    assert!(
        extract("save", empty.into_iter(), |_, value| Some(value))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn extraction_preserves_duplicate_errors_and_later_callbacks() {
    let duplicate: Arc<str> = Arc::from("duplicate");
    let input: Vec<(Arc<str>, Result<u64, io::Error>)> = vec![
        (Arc::from("before"), Ok(8)),
        (duplicate.clone(), Err(io::Error::other("first failure"))),
        (Arc::from("omitted"), Ok(5)),
        (duplicate.clone(), Err(io::Error::other("second failure"))),
        (Arc::from("after"), Ok(6)),
    ];
    let calls = RefCell::new(Vec::new());
    let failure = extract("save", input.into_iter(), |name, value| {
        calls.borrow_mut().push((name, value));
        (value % 2 == 0).then_some(value)
    })
    .unwrap_err();

    assert_eq!(
        calls.into_inner(),
        [
            (Arc::from("before"), 8),
            (Arc::from("omitted"), 5),
            (Arc::from("after"), 6),
        ]
    );
    assert_eq!(failure.op, "save");
    assert_eq!(failure.errors.0.len(), 2);
    assert!(Arc::ptr_eq(&failure.errors.0[0].0, &duplicate));
    assert!(Arc::ptr_eq(&failure.errors.0[1].0, &duplicate));
    assert_eq!(failure.errors.0[0].1.to_string(), "first failure");
    assert_eq!(failure.errors.0[1].1.to_string(), "second failure");
}
