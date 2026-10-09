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

`state_unit` verifies the actual `extract` loop for two bounded properties.
On normal return, the error case retains the original operation and the
exact ordered sequence of failed input `Arc<str>` names, including duplicates;
it returns `Ok` exactly when that sequence is empty. The reusable fold and
error-name projection are in `verification/specs/state_unit_extract.rs`.
For a successful return, the Vec contains exactly the ordered `Some` choices
from an input-length sequence: every failed input contributes `None`, and
each successful input's choice satisfies `f.ensures` for its name and value.
Thus callback results may vary between invocations; this is a relational
contract, not a purity or determinism claim. Its strength depends on the
caller's explicit `Fn` postcondition. Verified real-Vec/annotated-Fn witnesses
establish exact empty, Some/None, and ordered `[8, 6]` output cases.
Name equality here is Verus logical equality, not an `Arc::ptr_eq`
allocation-identity theorem.
The actual `save` callback expression is now factored into private
`saved_state_unit`, with a verified None/Some contract: None remains None;
Some produces a unit whose String has the same logical Unicode text as the
original `Arc<str>` and whose opaque blob is the original logical value.
A real Vec/that-function callback witness proves that two saved units remain
ordered around a filtered None, including exact name text and logical blob equality.
This helper is an active proof-enabling **twin-only source refactor**, tracked
for convergence in #16, not verification of unchanged upstream source. It
does not verify the async `save` invocation, `run_op`, or serialization.
The native Unicode/None/blob regression additionally exercises actual
encoding and parsing for two concrete saved states; that test is not a
universal serialization proof.

`SavedStateUnit` now uses `#[verifier::external_derive]` around its Protobuf
derive; its real fields remain transparent but generated serialization is
excluded. `verification/assumptions/saved_state_blob.rs` adds an opaque
external type specification with `external_body` **only on the type shell**:
`SavedStateBlob` has a private `ProtobufAny` field
(`vm/vmcore/src/save_restore.rs`), so transparent type support failed.
It grants no constructor, clone, bytes, parse, or encoding properties.
`verification/assumptions/arc_str_display.rs` adds one narrow trusted
stdlib contract connecting vstd's uninterpreted generic
`to_string_from_display_ensures::<Arc<str>>` to logical text equality for
every `Arc<str>` and resulting `String`. Both the native and Verus gates use
installed Rust 1.98.1 (`48a229ceaefd4985c50990b14116b6d856af0985`).
Its source shows Arc's
`Display` delegates to the inner `str` (`alloc/src/sync.rs`), `str::Display`
preserves text under ToString's default width and precision
(`core/src/fmt/mod.rs`), and `ToString` formats via Display
(`alloc/src/string.rs`). The unmodified `name.to_string()` could not prove
Unicode text equality with vstd's existing generic contract alone. This
library axiom is a new trusted boundary requiring independent review; no
target/helper behavior is assumed, and no pointer or byte-encoding claim
follows from it.

The production collection/match/return algorithm and private error fields
are unchanged; the one callback call is now bound to a local before the
original `if let` to ghost-record its returned choice. The private boundary
was narrowed from `IntoIterator` to
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
application to actual `run_op` output remain unproved. The theorems do not
prove callback invocation/effect history, error contents after
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
