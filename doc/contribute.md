# Contributing

## OpenVMM source

OpenVMM source is tracked directly under `openvmm` in this verified twin. A
normal clone includes it; no separate repository access or submodule
initialization is required.

## Test ownership

OpenVMM owns control-plane coverage. The phase-1 lifecycle and TTRPC interface
tests use OpenVMM's packaged guest artifacts. When invoked through NVX, the
custom TTRPC lifecycle, SMP, and snapshot test uses NVX's built kernel and
initramfs through the documented environment overrides. Run the unit and
documentation tests with `scripts/nvx.py test-openvmm-unit`, and use
`scripts/nvx.py test-openvmm --backend BACKEND` for the VMM tests.

NVX owns behavior that depends on its patched Linux kernel, Alpine userspace,
guest helpers, SMP behavior, or virtio devices. Add those scenarios to
`scripts/nvx_tools/microvm_tests.py` and run them with
`scripts/nvx.py test-microvm --backend BACKEND`. Keep guest workloads in
`scripts/nvx_tools/microvm_test_scripts` and retain complete failure logs.
