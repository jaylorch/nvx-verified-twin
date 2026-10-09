// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use vstd::prelude::*;

verus! {
    pub open spec fn valid_lapic_divide(divide: u32) -> bool {
        divide == 1 || divide == 2 || divide == 4 || divide == 8
            || divide == 16 || divide == 32 || divide == 64 || divide == 128
    }

    pub open spec fn lapic_ticks_post(
        downtime_ns: u64,
        apic_hz: u64,
        divide: u32,
        result: Option<u64>,
    ) -> bool {
        if !valid_lapic_divide(divide) {
            result.is_none()
        } else {
            let ticks = (downtime_ns as int * apic_hz as int)
                / (1_000_000_000int * divide as int);
            if ticks <= u64::MAX as int {
                match result {
                    Some(value) => value as int == ticks,
                    None => false,
                }
            } else {
                result.is_none()
            }
        }
    }
}
