// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use vstd::prelude::*;

verus! {
    /// Checks that the opaque error type is usable in a verified Result
    /// conversion and a collection, without asserting anything about its internals.
    pub fn collect_failure<E: Into<anyhow::Error>>(
        item: Result<u8, E>,
    ) -> (errors: Vec<anyhow::Error>)
        ensures errors.len() == if item.is_err() { 1int } else { 0int },
    {
        let mut errors = Vec::new();
        if let Err(error) = item {
            errors.push(error.into());
        }
        errors
    }
}
