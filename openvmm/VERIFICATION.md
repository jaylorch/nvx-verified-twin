# Verifying OpenVMM with Verus

This repository is configured to run Cargo Verus directly on the `openvmm` and
`state_unit` packages using the Verus source code in the sibling `verus`
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
```

The `openvmm` package depends on `vstd` and opts into Cargo Verus verification,
but it does not yet contain Verus specifications or proof annotations.

`state_unit` is also opted in, but **a successful package-level command does
not prove `extract`**. A direct `verus!` wrapper around the unchanged helper
currently fails: Verus ignores `StateTransitionError` and `UnitErrorSet`,
which are declared outside the macro, and reports `anyhow::Error` as an
unsupported type. A local experiment with external type specifications for
these types still failed because Verus does not support private fields in
transparent external type specifications (it suggested `external_body`, which
would introduce a trusted boundary). The iterator and `FnMut` proof obligations
were therefore not reached. No production helper replacement or trusted
assumption is retained.
