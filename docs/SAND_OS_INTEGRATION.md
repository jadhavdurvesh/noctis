# NOCTIS ↔ SAND OS integration

## Repositories inspected

- [SAND OS](https://github.com/jadhavdurvesh/sand-os) owns the kernel,
  process loader, syscall dispatcher and platform interfaces.
- [SAND Apps](https://github.com/jadhavdurvesh/sand-apps) owns the current
  SAPP v1 application packer, inspector, app workspace and format tests.

NOCTIS does not modify either repository. Its SAND target is kept in this
repository under `sand-app/`.

## Current application package

The integration workflow uses the current SAND Apps SAPP v1 toolchain, pinned
to commit `14ba9142e0c7822b5eac2ff29c7255b67795a1d8` for reproducible builds.
It does not use the older `sand-os/sdk/sapp` helper as a substitute for the
current `sand-apps` toolchain.

SAPP v1 is a native application container, not an ELF executable with a
different filename extension. The build pipeline is:

1. Assemble a freestanding 32-bit x86 entry point.
2. Link an ELF32/i386 executable with a dedicated SAPP linker script.
3. Use `sand-apps/tools/sapp_pack.py` to package the ELF as SAPP v1.
4. Use `sand-apps/tools/sapp_inspect.py` to verify the package.
5. Assert the architecture, segment count, entry address, permissions and
   relocation count in CI.
6. Upload the `.sapp` package, intermediate ELF, sidecar manifest and probe
   README as a GitHub Actions artifact.

The current NOCTIS probe produces one read/execute segment at
`0x40000000`. It does not create a writable-and-executable segment.

## Workflow

The workflow is
[`.github/workflows/sand-os-app.yml`](../.github/workflows/sand-os-app.yml).
It runs on changes to `sand-app/**`, on relevant pull requests, and by manual
dispatch. The artifact is named `noctis-sand-os-sapp-x86-32`.

Download it from the successful
[NOCTIS SAND OS SAPP build](https://github.com/jadhavdurvesh/noctis/actions/runs/42830887710).

Local build prerequisites are GCC with 32-bit output, GNU Binutils with i386
ELF support, GNU Make, Python 3.10+, and a checkout of SAND Apps:

```sh
make -C sand-app clean all SAND_APPS_SDK=../sand-apps
```

The generated files are:

- `sand-app/build/noctis-engine-probe.elf`
- `sand-app/build/noctis-engine-probe.sapp`

The sidecar `sand-app/manifest.json` describes the probe for NOCTIS-side
tooling and is included in the build artifact. The current SAND Apps packer
takes the executable name and version as command-line arguments and generates
its own SAPP JSON metadata; it does not ingest this sidecar manifest.

## What this proves—and what it does not

A successful CI run proves that the probe can be compiled as ELF32/i386,
packed as SAPP v1 by the pinned SAND Apps packer, and accepted by the matching
SAND Apps inspector. It does not prove that SAND OS currently offers a normal
third-party app installation or launch workflow.

The current SAND kernel's ring-3 syscall dispatcher accepts only a single
test-purpose operation: syscall number 1 with the token `0x53414E44` in EBX.
The probe invokes exactly that existing test operation and then idles. This
matches the current kernel's SAPP loader self-test; it is not a supported
general-purpose application syscall contract.

SAND OS does not yet expose the stable userspace APIs required by a browser:

- Window creation, window lifecycle and input events.
- Public framebuffer/graphics and text rendering services for applications.
- Console/terminal output and application-facing logging.
- File open/read/write, directories and persistent preferences.
- Networking, sockets, DNS and TLS.
- A stable process exit/wait and application lifecycle interface.

These interfaces must be designed, versioned and implemented in the SAND OS
repository before NOCTIS can become a functional graphical SAND application.
We must not guess syscall numbers, register conventions or kernel internals
from the test stub.

## Security and distribution status

The SAPP package is an unsigned development artifact. The SAND Apps format
documentation says signatures/trust-store verification are not implemented.
Do not treat the package as a production-signed app or distribute it as a
secure, installable browser. A production app installer and launcher also
remain future SAND OS work.

The SAND OS integration must remain a separate platform backend; no SAND
kernel source is copied into NOCTIS, and no changes are made to the SAND OS
kernel or SAND Apps repositories by this workflow.
