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

- Windows: `target/release/noctis-cli.exe`
- Linux/macOS: `target/release/noctis-cli`

Examples:

```sh
noctis-cli --help
noctis-cli --version
noctis-cli --tree examples/hello.html
noctis-cli --html examples/hello.html
noctis-cli --fragment --tree examples/hello.html
noctis-cli --css path/to/styles.css
```

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

- Windows x64: a ZIP containing `noctis-cli.exe`, README and license.
- Linux x64: a `.tar.gz` containing `noctis-cli`, README and license.
- macOS: a `.tar.gz` containing the native CLI executable, README and license.
- Android: a debug-signed APK with arm64-v8a, armeabi-v7a and x86_64 native
  parser libraries.
- A `SHA256SUMS` checksum file for the release assets.

Manual runs upload workflow artifacts. Tag runs additionally create/update a
GitHub Release and attach the packaged assets.

The desktop packages are CLI tools, not GUI browser installers. A Windows
`.exe` from this milestone is `noctis-cli.exe`. MSI/DMG/AppImage/DEB installers
and a graphical desktop browser shell are future packaging work.
