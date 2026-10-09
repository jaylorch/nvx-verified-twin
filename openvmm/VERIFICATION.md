# Verifying OpenVMM with Verus

This repository is configured to run Cargo Verus directly on the `openvmm`,
`state_unit`, and `virt` packages using the Verus source code in the sibling `verus`
directory.

## Set up external build packages

OpenVMM requires external build packages, including `protoc`, even when invoked
through Cargo Verus. Restore them once after cloning the repository:

```sh
cd openvmm
cargo xflowey restore-packages
```

The restore command may prompt for `sudo` on Linux while installing host
dependencies. Repeat it if OpenVMM's external build dependencies change.

## Set up the verification tools

Following the primary workflow in the
[Verus build instructions](../verus/BUILD.md), build Verus and `vstd` in one
shell. Perform these steps once after cloning the repository:

```sh
cd verus/source
./tools/get-z3.sh
source ../tools/activate
vargo build --release
```

The activation changes the shell environment for Verus development. Do not run
the verifier's Cargo command in that activated shell. Use a different,
non-activated shell for verification. Repeat this setup if the Verus source
changes.

## Run the verifier

Run Verus after updating files in the repository. From the repository root:

```sh
cd openvmm
PATH="../verus/source/target-verus/release:$PATH" \
  cargo verus verify -p openvmm --no-default-features

PATH="../verus/source/target-verus/release:$PATH" \
  cargo verus verify -p state_unit

PATH="../verus/source/target-verus/release:$PATH" \
  cargo verus verify -p virt
```

The `openvmm` package depends on `vstd` and opts into Cargo Verus verification,
but it does not yet contain Verus specifications or proof annotations.

`state_unit` verifies the actual `extract` loop for one bounded property:
on normal return, the error case retains the original operation and the
exact ordered sequence of failed input `Arc<str>` names, including duplicates;
it returns `Ok` exactly when that sequence is empty. The reusable fold and
error-name projection are in `verification/specs/state_unit_extract.rs`.
Name equality here is Verus logical equality, not an `Arc::ptr_eq`
allocation-identity theorem.
The production collection/match/return algorithm and private error fields
are unchanged. The private boundary was narrowed from `IntoIterator` to
`Iterator` and from `FnMut` to `Fn`, with explicit `.into_iter()` at the
`check` and `save` calls. This is a **modified-twin signature**, not a proof
of the unchanged upstream signature; issue #16 tracks convergence. Native
regression tests exercise callback effects after errors and real error
conversion, but no callback invocation trace is exported by the theorem.

The theorem requires `IteratorSpec::obeys_prophetic_iter_laws()` for the
particular iterator and the callback's `Fn::requires` for every input name
and success value; `Fn` alone does not imply purity or totality. A verified
`Vec::into_iter()`/`Fn` witness calls the actual loop, with concrete ordered
errors separated by a successful item and duplicate-name witnesses.
The actual `check` wrapper also verifies delegation to `extract`: on normal
return it yields `Ok(())` iff its initial iterator has no failed items;
otherwise it preserves `op` and the exact ordered failure names. The
`|_, _| Some(())` closure's callable obligation is discharged in that proof.
Precondition-free witnesses call the actual `check` with empty, ordered-error,
and duplicate-name `Vec::into_iter()` inputs.

`check` now takes a lawful `Iterator + IteratorSpec` rather than generic
`IntoIterator` because the latter supplies no general conversion contract
for the resulting iterator's sequence and prophetic laws. Three existing
`Vec` call sites use `.into_iter()` explicitly. Existing mapped iterators in
`advance_time` and restore remain streaming, without collection or added
allocation. This is another **modified-twin private signature**; issue #16
tracks convergence for both narrowed boundaries. The actual reset, restore,
advance-time, and save caller obligations are **not** verified: in particular
the mapped-iterator and `run_op` contracts and the `save` callback's
preconditions and saved blob relation remain unproved. The theorems do not
prove a result vector, callback invocation history, error contents after
`Into<anyhow::Error>`, termination, panic freedom, or snapshot correctness.

The `anyhow::Error` type barrier is removed by the
opaque external type specification in
`verification/assumptions/anyhow_error.rs`. Its trusted source is anyhow
1.0.99 (`src/lib.rs` and `src/error.rs`); Verus cannot inspect the private
representation of that external crate, and a transparent external type
specification fails on private fields. Opacity provides *only* type support:
no error contents, constructors, conversion results, or panic freedom are
assumed. The separate verified case in
`verification/specs/anyhow_error_example.rs` conditionally collects a real
`anyhow::Error` through a generic `Into` conversion and proves the resulting
error count (1 verified, 0 errors). This uses vstd's existing generic `Into`
contract; it does not establish that arbitrary user-defined conversions
cannot panic, terminate, or safely unwind. Its postcondition applies on normal
return. The example is not the production `extract` implementation, and its
`verus_only` module is not exercised by ordinary package tests.

`StateTransitionError` and `UnitErrorSet` are inside `verus!` and remain
transparent with their private fields and real constructor: no external type
specification or assumption was added for either type.
`#[verifier::external_derive]` on `StateTransitionError` excludes its
generated Debug, Error, and Display implementations from verification,
leaving their Rust behavior untouched. This is needed because thiserror's
derived formatting invokes unsupported functions, and Verus does not
recognize the generated `std::error::Error` trait implementation.

The `Iterator` narrowing avoids an uncontracted generic
`IntoIterator::into_iter` step inside the proof. No `external_body` on
`extract`, blanket iterator axiom, or substitute extraction body was added.

## Verified LAPIC tick conversion

`virt::time_abi::rate::lapic_ticks` has a postcondition in
`verification/specs/lapic_ticks.rs`. For all input values, it returns `None`
for divides outside `{1, 2, 4, 8, 16, 32, 64, 128}`. Otherwise it returns
`floor(downtime_ns * apic_hz / (1_000_000_000 * divide))` when that mathematical
value fits `u64`, and `None` when it does not. The proof also discharges the
production arithmetic's intermediate overflow and division-by-zero obligations.
The production body is unchanged apart from erased proof blocks.

The reusable `lapic_ticks_fit_backend_rate` lemma proves that a valid divide
and an LAPIC rate at most 1 GHz always yield `Some(ticks)` with
`ticks <= downtime_ns`, for every `u64` downtime. This establishes the
arithmetic no-overflow condition for the supported backend rates. The lemma
does not yet verify that the snapshot caller supplies those validated inputs.

The additional trusted boundary is the generic `u32::is_power_of_two` contract
in `verification/assumptions/u32_is_power_of_two.rs`, necessary because the
vendored `vstd` does not specify this method. It models Rust's standard-library
method over all `u32` values; it does not assume the LAPIC postcondition.
Existing `vstd` contracts for integer conversions and `Result::ok` remain
trusted. The target function has no `external_body` annotation.

This proves an arithmetic component used by snapshot time repair, not the
snapshot coordination or save/restore equivalence property. Caller validation,
clock accuracy, and application of the resulting ticks to device state remain
outside this proof.
