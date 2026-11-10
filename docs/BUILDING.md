# Building and Packaging NOCTIS

NOCTIS currently ships a command-line engine lab and an Android parser-preview
application. These are developer tools for the independently implemented
HTML/CSS subsystems; neither is a full graphical browser.

## Requirements

- Stable Rust toolchain (see `rust-toolchain.toml`).
- For Android: JDK 17, Android SDK platform 35, Android NDK 27.2.12479018,
  Gradle 8.9, and `cargo-ndk` 4.1.2.

## Desktop CLI

Build on the host operating system:

```sh
cargo build --release -p noctis-cli
```

The executable is written to:

- Windows: `target/release/noctis-cli.exe` and `target/release/noctis-browser.exe`
- Linux/macOS: `target/release/noctis-cli` and `target/release/noctis-browser`

Examples:

```sh
noctis-cli --help
noctis-cli --version
noctis-cli --tree examples/hello.html
noctis-cli --html examples/hello.html
noctis-cli --fragment --tree examples/hello.html
noctis-cli --css path/to/styles.css
```

## NOCTIS terminal browser alpha

Build:

```sh
cargo build --release -p noctis-browser
```

Run:

```sh
noctis-browser https://example.com
# or launch without an argument to get an address prompt
noctis-browser
```

Bare hostnames default to HTTPS. In the browser prompt, enter a numbered link
to follow it, a URL/hostname to navigate, `b` / `f` for history,
`r` to reload, `h` for help, or `q` to quit.

The app currently displays readable terminal text using NOCTIS HTML parsing.
It supports HTML and plain-text responses, HTTP/HTTPS only, an eight-redirect
limit, a 20-second timeout and a 5 MiB response cap. It does not yet provide
JavaScript, CSS layout, forms, cookies, browser storage, binary downloads or
a graphical window. See [Browser Alpha](BROWSER_ALPHA.md).

`--css` prints the parsed rule structures from NOCTIS's current CSS subset.
The CLI does not open network URLs, execute JavaScript, render pages, or
provide a graphical window.

## Android Engine Lab APK

The Android app is a native Kotlin shell calling the NOCTIS HTML parser through
the project's Rust JNI bridge. It does not use Android WebView or embed another
browser engine. The preview includes an HTML text editor, a parse action, and
the resulting NOCTIS DOM tree.

On a machine with the Android prerequisites installed:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk --version 4.1.2 --locked

cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -o android/app/src/main/jniLibs \
  build --release -p noctis-android-bridge

gradle --no-daemon -p android :app:assembleDebug
```

APK output:

`android/app/build/outputs/apk/debug/app-debug.apk`

This is a debug-signed APK for development and testing. It is not a
production/Play Store release and does not have production signing or update
infrastructure.

## SAND OS native SAPP package

NOCTIS has a separate SAND OS loader-probe target. It builds a freestanding
ELF32/i386 executable and packages it as the SAND-native **SAPP v1** container
using the current, pinned SAND Apps toolchain. It does not rename an ELF file
to `.sapp`, and it does not modify the SAND OS kernel or SAND Apps repository.

For a local build, install GNU Make, GCC with 32-bit output, GNU Binutils with
i386 ELF support, Python 3.10+, and clone `jadhavdurvesh/sand-apps` beside
the NOCTIS checkout. Then run:

```sh
make -C sand-app clean all SAND_APPS_SDK=../sand-apps
```

The outputs are:

- `sand-app/build/noctis-engine-probe.elf` — intermediate ELF32/i386.
- `sand-app/build/noctis-engine-probe.sapp` — native SAPP v1 development package.

The workflow
`.github/workflows/sand-os-app.yml` runs on probe/workflow changes, pull
requests and manual dispatch. It checks out the SAND Apps tools at commit
`14ba9142e0c7822b5eac2ff29c7255b67795a1d8`, builds the probe, validates the
container with SAND Apps' own inspector, and uploads the `.sapp` as a
30-day GitHub Actions artifact named `noctis-sand-os-sapp-x86-32`.

This is a loader/format probe only. The present SAND OS userspace ABI exposes
only a test syscall; it has no public windowing, graphics, files, console or
networking APIs for a browser. The package is unsigned and is not a production
installable browser. Read [SAND OS integration](SAND_OS_INTEGRATION.md)
before treating it as a runtime application.

## GitHub Actions

### Continuous verification

- `.github/workflows/ci.yml` checks Rust formatting, Clippy and tests on
  Ubuntu and Windows.
- `.github/workflows/rust-maintenance.yml` formats Rust code, resolves the
  workspace lockfile, verifies the build and tests, and commits formatting or
  lockfile changes to `main`.
- `.github/workflows/android-preview.yml` builds the Android Engine Lab APK
  for arm64-v8a, armeabi-v7a and x86_64 when relevant files change, and uploads
  the APK as a 30-day workflow artifact.

### Desktop and tagged releases

`.github/workflows/release.yml` can be run manually from the Actions tab or
automatically by pushing a version tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

It produces:

- Windows x64: a ZIP containing `noctis-cli.exe`, `noctis-browser.exe`, README and license.
- Linux x64: a `.tar.gz` containing `noctis-cli`, `noctis-browser`, README and license.
- macOS: a `.tar.gz` containing the native CLI executable, README and license.
- Android: a debug-signed APK with arm64-v8a, armeabi-v7a and x86_64 native
  parser libraries.
- A `SHA256SUMS` checksum file for the release assets.

Manual runs upload workflow artifacts. Tag runs additionally create/update a
GitHub Release and attach the packaged assets.

The desktop packages are CLI tools, not GUI browser installers. A Windows
`.exe` from this milestone is `noctis-cli.exe`. MSI/DMG/AppImage/DEB installers
and a graphical desktop browser shell are future packaging work.
