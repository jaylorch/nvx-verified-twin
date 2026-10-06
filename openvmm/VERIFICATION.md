# Verifying OpenVMM with Verus

This repository is configured to verify the `openvmm` package using the Verus
source code in the sibling `verus` directory.

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
```

The package depends on `vstd` and opts into Cargo Verus verification, but it
does not yet contain Verus specifications or proof annotations.
