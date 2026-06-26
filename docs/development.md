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

## Verification Checklist

Use this baseline before handing off functional changes:

```powershell
cargo test
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

The MSI installer uses:

- `app/src-tauri/icons/icon.ico` for product and shortcut icons.
- `app/src-tauri/icons/wix-banner.bmp` for the top installer banner.
- `app/src-tauri/icons/wix-dialog.bmp` for the welcome and finish dialog image.
- `app/src-tauri/wix/main.wxs` as the custom WiX template.

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
