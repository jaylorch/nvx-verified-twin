// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;
use vstd::prelude::*;
use vstd::string::to_string_from_display_ensures;

verus! {
    // Trusted sources: installed Rust 1.98.1 (48a229ceaefd4985c50990b14116b6d856af0985),
    // alloc/src/sync.rs:3722-3725, Arc<T>::Display delegates to &**self;
    // core/src/fmt/mod.rs:2965-2968, str::Display preserves text under
    // ToString's default width and precision;
    // alloc/src/string.rs:2918-2945, ToString formats Display.
    // vstd/string.rs leaves generic ToString's display relation uninterpreted.
    // This axiom connects only Arc<str>'s formatted Unicode text to its view;
    // it grants no byte-encoding, pointer, or arbitrary-Display property.
    pub broadcast axiom fn arc_str_display_text(name: &Arc<str>, rendered: String)
        ensures
            #[trigger] to_string_from_display_ensures::<Arc<str>>(name, rendered)
                ==> rendered@ == (**name)@,
    ;
}
