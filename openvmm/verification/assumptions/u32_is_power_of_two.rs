// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(unsafe_code)]

use vstd::prelude::*;

verus! {
    // Trusted source: Rust's u32::is_power_of_two documentation and
    // core/num/uint_macros.rs (`self.count_ones() == 1`).
    // Verus/vstd does not currently specify this standard-library method.
    // For every u32, exactly one set bit iff x != 0 and x & (x - 1) == 0.
    pub assume_specification [u32::is_power_of_two](x: u32) -> (result: bool)
        ensures result == (x != 0 && (x & ((x as int - 1) as u32)) == 0);
}
