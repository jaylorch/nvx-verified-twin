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

    pub proof fn lapic_ticks_fit_backend_rate(
        downtime_ns: u64,
        apic_hz: u64,
        divide: u32,
        result: Option<u64>,
    )
        requires
            apic_hz <= 1_000_000_000,
            valid_lapic_divide(divide),
            lapic_ticks_post(downtime_ns, apic_hz, divide, result),
        ensures
            result.is_some(),
            match result {
                Some(ticks) => ticks <= downtime_ns,
                None => false,
            },
    {
        use vstd::arithmetic::div_mod::{lemma_div_is_ordered, lemma_div_multiples_vanish};
        use vstd::arithmetic::mul::{lemma_mul_inequality, lemma_mul_is_commutative};

        let denominator = 1_000_000_000int * divide as int;
        let product = downtime_ns as int * apic_hz as int;
        assert(apic_hz as int <= denominator);
        lemma_mul_inequality(apic_hz as int, denominator, downtime_ns as int);
        lemma_mul_is_commutative(downtime_ns as int, apic_hz as int);
        lemma_div_is_ordered(product, denominator * downtime_ns as int, denominator);
        lemma_div_multiples_vanish(downtime_ns as int, denominator);
    }
}
