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

Build the native CLI release binary:

```powershell
cargo build -p neopad-cli --release
```

Build the desktop app and MSI:

```powershell
pnpm tauri:build
```

The root `tauri:build` script intentionally performs three steps:

1. Build the `neopad-cli` release binary.
2. Run `scripts/prepare-cli-sidecar.mjs` to copy it to the target-specific
   sidecar name required by Tauri (`neopad-cli-<triple>[.exe]`). The script
   reads the Rust target triple from `CARGO_BUILD_TARGET` (set by CI for
   cross-compilation) or falls back to the host triple.
3. Build the Tauri app and platform bundle.

Use the root command for distributable builds so the installed app can deploy
the CLI and offer status and repair actions in Settings > Agent tools.

Version 0.7.1 replaces the prompt library with ordinary project notes; see
[Projects and migration](projects.md). Version 0.7.0 removes the former MCP sidecar. Existing Agent connections must
follow [MCP-to-CLI migration](cli.md#migrate-from-mcp). Before preparing a build,
keep the Cargo workspace, root and frontend packages, Tauri config, and Cargo
lockfile in sync, and record the version in `CHANGELOG.md`.

## Desktop End-to-End Tests

The Windows desktop suite drives the compiled Tauri WebView2 application with
`tauri-driver` and WebdriverIO. Test data is isolated under
`target/e2e-workspace`; it never uses the normal `~/.neopad` workspace.

Install or refresh the matching drivers and run the suite:

```powershell
pnpm test:e2e
```

The setup script installs the pinned `tauri-driver` 2.1.0 and
`msedgedriver-tool` when missing.
The Microsoft Edge driver is downloaded to `target/tools/`, matching the local
WebView2 runtime. The suite then builds `neopad-app.exe` without an installer
bundle and exercises note creation, autosave across tab switches, and settings.

On Windows, `NEOPAD_E2E=1` builds the test window with native WebView2 API
options: remote debugging uses an ephemeral port, and both the app and
WebDriver use `target/e2e-workspace/webview2-profile`. WebView2 150 and newer
ignore environment overrides in elevated processes, including hosted Windows
CI runners; using the native API avoids losing the debugging switch or
waiting for `DevToolsActivePort` in a different profile. See
[Microsoft's explanation](https://github.com/MicrosoftEdge/WebView2Feedback/issues/5640#issuecomment-4923662109).
Ordinary startup retains the automatic window and has no test debugging switch.

The Explorer test checks both the opened directory and the selected file for
Unicode and punctuated paths. Native reveal calls pass an explicit parent PIDL
and relative child PIDL to the Windows Shell. On failure the suite logs the
Explorer session, folder contents, and selected items. Mermaid assertions check
SVG label text and geometry because WebDriver text extraction varies by runtime.

Relevant files:

```text
app/src/composables/*.test.ts  Document, lifecycle, and preference state tests
app/src/lib/*.test.ts          Autosave, shortcuts, search, editor, and text tests
app/src/lib/note-export.test.ts PDF pagination boundary tests
cli/tests/commands.rs         CLI process, concurrency, conflict, and deployment tests
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
pass `pnpm audit:prod` and `cargo audit`.

The root pnpm KaTeX override keeps the Markdown plugin and Mermaid on the
patched version used by the frontend's KaTeX styles. Keep it aligned with
`app/package.json`; the desktop suite verifies ordinary formulas and Mermaid
math labels after dependency changes. The production audit checks exact
installed versions without advisory exclusions.

For CLI-only changes, also run:

```powershell
cargo build -p neopad-cli --release
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

The native CLI is packaged through Tauri `externalBin`. The MSI also installs
`neopad-cli.exe` through the custom WiX template. Desktop startup deploys it to
`~/.neopad/.agents/bin/`; Settings > Agent tools checks or repairs it.
Workspace initialization also creates the root `AGENTS.md` only when absent.
Edit the default template in `crates/neopad-core/resources/workspace-AGENTS.md`;
existing user guides are never upgraded automatically. Settings opens the guide
as an external Markdown document.

The Windows MSI installer uses:

- `app/src-tauri/icons/icon.ico` for product and shortcut icons.
- `app/src-tauri/icons/wix-banner.bmp` for the top installer banner.
- `app/src-tauri/icons/wix-dialog.bmp` for the welcome and finish dialog image.
- `app/src-tauri/wix/main.wxs` as the custom WiX template.
- `app/src-tauri/wix/locales/en-US.wxl` for English installer strings and a
  code page that can also store the bilingual Chinese-default option.
- `target/release/neopad-cli.exe` as the native CLI sidecar source.

The MSI pages use `en-US`. The custom language page is English except for its
bilingual Chinese-default option, and lets the user select Chinese or English.
The installer writes
that choice to `HKCU\Software\NeoPad\NeoPad\InstallLanguage`; only a new default
workspace imports it, so upgrades do not overwrite an existing app language
preference. The English locale uses code page 936 so the same MSI database can
store the Chinese option without producing mojibake.

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
`pnpm audit:prod` through npm's Bulk Advisory API. A separate CI job installs
the locked `cargo-audit 0.22.2`
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
