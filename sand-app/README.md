# NOCTIS on SAND OS — SAPP loader probe

This directory builds a minimal **SAPP v1** application for SAND OS using the
current packer and inspector from the separate `jadhavdurvesh/sand-apps`
repository.

It is an integration probe, not the NOCTIS browser port. The current SAND OS
kernel has no stable public userspace API for windows, framebuffer drawing,
terminal I/O, files, networking, or application lifecycle. Its only accepted
ring-3 syscall is a test operation (EAX=1, EBX=0x53414E44). The probe makes
that call and then idles, matching the kernel's existing SAPP loader test.

## Build

Requirements: GCC with 32-bit code generation, GNU Binutils/i386 linker, GNU
Make, Python 3.10+, and a checkout of `jadhavdurvesh/sand-apps`.

From this directory:

```sh
make clean all SAND_APPS_SDK=../sand-apps
```

Outputs:

- `build/noctis-engine-probe.elf` — intermediate freestanding ELF32/i386.
- `build/noctis-engine-probe.sapp` — SAPP v1 development package.

The build uses `sand-apps/tools/sapp_pack.py` and validates the result with
`sand-apps/tools/sapp_inspect.py`. The GitHub Actions workflow pins the
SAND Apps toolchain checkout to a known commit for reproducible builds.

## Compatibility boundaries

The package targets x86-32, the current SAPP v1 architecture. The linker
creates one read/execute segment at 0x40000000; it does not create a writable
executable segment. The app has no declared capabilities and does not attempt
to call undocumented syscalls.

SAPP signatures and the production trust store are not implemented. This is
an unsigned development artifact. The current kernel also does not expose a
normal filesystem package-install or launch path for third-party apps. A
successful host-side pack/inspect workflow proves format validity, not that a
user can install or use a graphical browser in SAND OS.

A real NOCTIS browser application needs versioned SAND OS services for at
least windowing/input, text and graphics, files, process exit, and eventually
network access. Those ABI additions belong in the SAND OS repository first.
