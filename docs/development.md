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

1. Build `neopad-mcp.exe` in release mode.
2. Run `scripts/prepare-mcp-sidecar.ps1` to copy it to the target-specific
   sidecar name required by Tauri.
3. Build the Tauri app and MSI.

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
app/src/lib/autosave.test.ts  Frontend autosave unit tests
mcp-server/tests/http.rs      MCP HTTP child-process protocol tests
e2e/wdio.conf.ts              Desktop WebDriver configuration
e2e/specs/                    Desktop interaction specifications
scripts/setup-e2e.ps1         Windows driver setup
```

## Verification Checklist

Use this baseline before handing off functional changes:

```powershell
cargo test
pnpm test:frontend
pnpm build
pnpm tauri:build
```

For MCP-only changes, also run:

```powershell
cargo build -p neopad-mcp --release
```

## Packaging Notes

The app currently targets MSI only:

```json
"targets": ["msi"]
```

NSIS is intentionally not part of the current build path.

The Windows release executable uses the Windows GUI subsystem so it does not
open a console window on launch.

The MCP service is packaged as a sidecar through Tauri `externalBin`. The MSI
also installs `neopad-mcp.exe` through the custom WiX template so installed
builds can resolve the service binary reliably.

The MSI installer uses:

- `app/src-tauri/icons/icon.ico` for product and shortcut icons.
- `app/src-tauri/icons/wix-banner.bmp` for the top installer banner.
- `app/src-tauri/icons/wix-dialog.bmp` for the welcome and finish dialog image.
- `app/src-tauri/wix/main.wxs` as the custom WiX template.
- `target/release/neopad-mcp.exe` as the MCP sidecar source.

When modifying installer UI, keep WiX default text areas clear. The dialog image
is a full background for welcome and finish pages, not a standalone logo slot.

## Runtime Workspace

Development and installed builds both initialize the default workspace:

```text
~/.neopad/
```

Delete or move that directory if you need a clean first-run state. Do not use
test code that writes outside a temporary workspace unless explicitly testing
the default path.
