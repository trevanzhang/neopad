# Development

## Toolchain

Required:

- Node.js
- pnpm
- Rust stable with Cargo
- Tauri 2 system dependencies

Windows MSI packaging additionally requires WiX. On this machine WiX 3.14 has
been staged from:

```text
C:\Users\ferri\Downloads\wix314-binaries.zip
```

to:

```text
C:\Users\ferri\AppData\Local\tauri\WixTools314
```

## Install Dependencies

```powershell
pnpm install
```

## Common Commands

Run the Vue frontend only:

```powershell
pnpm dev
```

Run the Tauri desktop app in development mode:

```powershell
pnpm tauri:dev
```

Build the frontend:

```powershell
pnpm build
```

Run Rust tests:

```powershell
cargo test
```

Run frontend unit tests:

```powershell
pnpm test:frontend
```

Run all fast tests:

```powershell
pnpm test:all
```

Build all Rust crates:

```powershell
cargo build
```

Build the MCP server release binary:

```powershell
cargo build -p neopad-mcp --release
```

Build the desktop app and MSI:

```powershell
pnpm tauri:build
```

The root `tauri:build` script intentionally performs three steps:

1. Build the `neopad-mcp` release binary.
2. Run `scripts/prepare-mcp-sidecar.mjs` to copy it to the target-specific
   sidecar name required by Tauri (`neopad-mcp-<triple>[.exe]`). The script
   reads the Rust target triple from `CARGO_BUILD_TARGET` (set by CI for
   cross-compilation) or falls back to the host triple.
3. Build the Tauri app and platform bundle.

Use the root command for distributable builds so the installed app can start
the MCP service from Settings.

## Desktop End-to-End Tests

The Windows desktop suite drives the compiled Tauri WebView2 application with
`tauri-driver` and WebdriverIO. Test data is isolated under
`target/e2e-workspace`; it never uses the normal `~/.neopad` workspace.

Install or refresh the matching drivers and run the suite:

```powershell
pnpm test:e2e
```

The setup script installs `tauri-driver` and `msedgedriver-tool` when missing.
The Microsoft Edge driver is downloaded to `target/tools/`, matching the local
WebView2 runtime. The suite then builds `neopad-app.exe` without an installer
bundle and exercises note creation, autosave across tab switches, and settings.

Relevant files:

```text
app/src/composables/*.test.ts  Document, lifecycle, and preference state tests
app/src/lib/*.test.ts          Autosave, shortcuts, search, editor, and text tests
app/src/lib/note-export.test.ts PDF pagination boundary tests
mcp-server/tests/http.rs       MCP HTTP child-process protocol tests
e2e/wdio.conf.ts               Desktop WebDriver configuration
e2e/specs/                     Desktop interaction specifications
scripts/setup-e2e.ps1          Windows driver setup
```

## Verification Checklist

Use this baseline before handing off functional changes:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm test:frontend
pnpm build
pnpm check:versions
```

Run `pnpm test:e2e` for desktop workflow changes and `pnpm tauri:build` when
packaging or sidecar behavior changes. Production dependency changes must also
pass `pnpm audit --prod` and `cargo audit`.

For MCP-only changes, also run:

```powershell
cargo build -p neopad-mcp --release
```

## Packaging Notes

The root build script selects explicit bundles for the current platform:

```text
Windows  msi
macOS    dmg
Linux    deb,appimage
```

`tauri.conf.json` uses MSI as its safe direct-build default. The root script
overrides it on macOS and Linux. NSIS is intentionally not part of the current
build path; do not use `targets: "all"` because that also enables NSIS on
Windows.

The Windows release executable uses the Windows GUI subsystem so it does not
open a console window on launch.

The MCP service is packaged as a sidecar through Tauri `externalBin`. The MSI
also installs `neopad-mcp.exe` through the custom WiX template so installed
builds can resolve the service binary reliably.

The Windows MSI installer uses:

- `app/src-tauri/icons/icon.ico` for product and shortcut icons.
- `app/src-tauri/icons/wix-banner.bmp` for the top installer banner.
- `app/src-tauri/icons/wix-dialog.bmp` for the welcome and finish dialog image.
- `app/src-tauri/wix/main.wxs` as the custom WiX template.
- `target/release/neopad-mcp.exe` as the MCP sidecar source.

When modifying installer UI, keep WiX default text areas clear. The dialog image
is a full background for welcome and finish pages, not a standalone logo slot.

## Cross-Platform Builds

Pull requests compile the complete Rust workspace on Linux x64 and macOS ARM64
in addition to the Windows test and desktop E2E jobs. Cross-platform release
builds run automatically in GitHub Actions when a `v*` tag is pushed (see
`.github/workflows/release.yml`). The matrix covers:

- Windows x64 (`x86_64-pc-windows-msvc`) producing `.msi`
- macOS ARM64 (`aarch64-apple-darwin`) producing `.dmg`
- Linux x64 (`x86_64-unknown-linux-gnu`) producing `.deb` and `.AppImage`

Linux builds require these system packages:

```text
libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
libssl-dev libgtk-3-dev libayatana-appindicator3-dev
```

macOS builds cross-compile for Apple Silicon from the `macos-latest` (ARM64)
runner using `--target aarch64-apple-darwin`.

CI and release preflight both reject known production npm vulnerabilities with
`pnpm audit --prod`. A separate CI job installs the locked `cargo-audit 0.22.2`
release and rejects RustSec advisories; release preflight repeats that Rust
dependency audit. The release workflow also runs version/tag validation,
formatting, Clippy, tests, and the frontend build. Each platform must produce
its complete expected artifact set, and SHA-256 manifests are attached to a
draft Release. The draft must not be published until Windows Authenticode and
macOS signing/notarization have been verified with project-owned credentials.

The two `quick-xml 0.39.4` advisories listed in `.cargo/audit.toml` are narrowly
ignored only for `wayland-scanner 0.31.10`: the proc-macro parses protocol XML
shipped by Wayland crates during compilation and has no runtime or user-input
path. The runtime plist path uses `plist 1.10.0` and `quick-xml 0.41.0`. Remove
the ignores as soon as `wayland-scanner` publishes a compatible update.

Known platform gaps: window opacity and autostart are no-ops on macOS and
Linux. Both compile cleanly but have no effect outside Windows.

## Runtime Workspace

Development and installed builds both initialize the default workspace:

```text
~/.neopad/
```

Delete or move that directory if you need a clean first-run state. Do not use
test code that writes outside a temporary workspace unless explicitly testing
the default path.
