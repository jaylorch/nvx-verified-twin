// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use vstd::prelude::*;

verus! {
    // Trusted source: anyhow 1.0.99, src/lib.rs (Error owns a private
    // Own<ErrorImpl>) and src/error.rs (its constructors and conversions).
    // Verus cannot inspect this external crate's private representation;
    // a transparent external type specification rejects the private fields.
    // This only makes Error an opaque value usable in verified containers.
    // It specifies no constructor, conversion, error content, or panic behavior.
    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExAnyhowError(anyhow::Error);
}
