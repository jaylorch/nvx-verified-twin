// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use vstd::prelude::*;

verus! {
    // Trusted source: vmcore::save_restore::SavedStateBlob wraps a private
    // ProtobufAny field (vm/vmcore/src/save_restore.rs). Verus cannot inspect
    // that representation. This grants only opaque logical identity; it
    // specifies no constructors, bytes, decoding, or serialization behavior.
    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExSavedStateBlob(vmcore::save_restore::SavedStateBlob);
}
