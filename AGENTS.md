# AGENTS.md

This guide is for coding agents working in this repository.

## Project Summary

NeoPad is a local-first desktop note pad built with Tauri 2, Vue 3,
TypeScript, and Rust. It stores user notes as Markdown files and exposes the
same local data through a standalone MCP server.

Keep NeoPad small and fast. It is a capture tool, not a knowledge-base suite.

## Repository Layout

```text
app/                  Vue frontend and Tauri desktop app
app/src/              Vue components, styles, TypeScript helpers
app/src-tauri/        Tauri Rust shell, commands, tray, hotkeys, packaging
crates/neopad-core/   Shared Rust core for workspace and note operations
mcp-server/           Standalone MCP local HTTP server
docs/                 Human-facing project docs
```

## Important Rules

- Use `neopad-core` for all note, workspace, path, search, and write behavior.
- Do not duplicate filesystem access logic in the Tauri shell or MCP server.
- Note content must stay in `~/.neopad/notes/*.md`.
- Metadata belongs in `config.json`, `meta/tabs.json`, and
  `meta/reminders.json`. The reminder file stores delivery state only; reminder
  content remains in Markdown notes.
- Never physically delete user notes. Move notes to `trash/`.
- Keep path safety strict. Reject absolute paths, `..`, and file names that
  escape the workspace.
- MCP HTTP service must be off by default in the desktop app.
- When MCP is enabled, local agents with the bearer token can read and write
  notes.
- MCP HTTP requests must require `Authorization: Bearer <token>`.
- Browser-originated MCP requests must pass local Origin validation.
- MCP diagnostics and startup errors must go to stderr, not protocol responses.
- Do not introduce cloud sync, accounts, RAG, vector search, backlinks, or an AI
  chat panel unless the task explicitly changes scope.

## Build Commands

Run these from the repository root.

```powershell
cargo test
pnpm build
pnpm tauri:build
```

For MCP changes:

```powershell
cargo build -p neopad-mcp --release
```

For Rust formatting:

```powershell
cargo fmt
```

## Git Workflow

- Do not make code changes directly on `main`.
- Before modifying files, check the current branch and working tree.
- If the current branch is `main`, create a dedicated branch using the
  `codex/` prefix before making changes.
- Use a concise branch name that describes the task, for example
  `codex/fix-reminder-dark-theme`.
- Preserve existing uncommitted user changes. Do not discard, reset, or
  overwrite them.
- Do not commit, push, or create a pull request unless explicitly requested.
- Run the relevant validation commands before handing off changes.

## Versioning

- Follow Semantic Versioning for distributable builds.
- Keep the workspace version in `Cargo.toml`, the root `package.json`, the
  frontend version in `app/package.json`, and the Tauri version in
  `app/src-tauri/tauri.conf.json` synchronized. Update `Cargo.lock` when the
  workspace version changes.
- Verify version sync with `pnpm check:versions` before committing a release.
- Do not bump versions for routine development commits. Bump the version when
  preparing a distinct distributable build or release.
- Use a patch bump for compatible fixes, a minor bump for backward-compatible
  features, and a major bump for incompatible changes.
- Use prerelease labels such as `beta` only when the task explicitly establishes
  a prerelease channel. Do not add them to ordinary development builds.

## Release Process

Releases are built automatically by GitHub Actions when a `v*` tag is pushed.

1. Ensure `main` and `develop` are in sync and all CI checks pass.
2. Bump versions (see Versioning above), commit to `develop`, and fast-forward
   merge into `main`.
3. Tag the release on `main` and push:

   ```text
   git tag v0.x.x
   git push origin v0.x.x
   ```

4. The `Release` workflow builds all three platforms in parallel and uploads
   installers to the GitHub Release for that tag:

   ```text
   Windows x64   .msi     (windows-latest)
   macOS ARM64   .dmg     (macos-latest, --target aarch64-apple-darwin)
   Linux x64     .deb     (ubuntu-22.04)
   Linux x64     .AppImage (ubuntu-22.04)
   ```

5. After the workflow finishes, edit the Release to set the title and notes.
   The CI-generated Release starts with a default title and no notes.
6. Update `CHANGELOG.md`: move entries from `## Unreleased` into a dated
   `## x.y.z` section if not already done.

Do not delete or recreate published tags. If a release needs rebuilding, delete
the GitHub Release and re-push the tag.

## Cross-Platform Packaging Notes

The app bundles for all platforms (`"targets": "all"` in `tauri.conf.json`):

- Windows: `.msi` (WiX)
- macOS: `.dmg`
- Linux: `.deb` and `.AppImage`

Do not re-enable NSIS unless the task is specifically about NSIS packaging.

### Windows (WiX)

The MSI installer uses a custom WiX template:

```text
app/src-tauri/wix/main.wxs
```

The installer images must remain exactly:

```text
app/src-tauri/icons/wix-banner.bmp   493 x 58
app/src-tauri/icons/wix-dialog.bmp   493 x 312
```

The dialog image is used as a full background on WiX welcome and finish pages.
Keep the right-side text area clear so default WiX title and body text remain
readable.

Desktop and start-menu shortcuts should both use `ProductIcon` and
`System.AppUserModel.ID`.

### macOS

macOS builds target Apple Silicon (`aarch64-apple-darwin`). The `.icns` icon
at `app/src-tauri/icons/icon.icns` must be kept in sync with the source logo.

Window opacity and autostart are no-ops on macOS (functionality is Windows-only
for now). Both compile cleanly.

### Linux

Linux builds require `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`,
`librsvg2-dev`, and related system packages. The CI workflow installs these
automatically; see `.github/workflows/release.yml`.

Window opacity and autostart are no-ops on Linux (same as macOS).

### MCP Sidecar

The MCP server is bundled as a Tauri `externalBin` sidecar.
`scripts/prepare-mcp-sidecar.mjs` copies the release binary to the
target-triple-suffixed name Tauri expects. The `tauri:build` script runs this
automatically. Do not call the PowerShell `prepare-mcp-sidecar.ps1` directly;
it is retained only for legacy Windows-local workflows.

## Frontend Guidelines

- Keep the UI compact and utility-focused.
- Do not add a landing page.
- Prefer existing component and CSS patterns.
- Keep text within fixed UI controls from overflowing.
- Use the existing logo assets from `app/src/assets/`.
- Avoid decorative-only redesigns unless the task is specifically visual.

## Core Data Guidelines

- Prefer focused core functions with tests over UI-side workarounds.
- Preserve atomic write behavior.
- Preserve `expectedUpdatedAt` conflict checks for full-page MCP updates.
- Pinned default pages such as Inbox and Clipboard must not be deletable.

## Documentation Guidelines

- Keep Markdown files UTF-8.
- Prefer ASCII unless non-ASCII content is required.
- Update `README.md` and `docs/` when commands, packaging, or user-visible
  behavior changes.
- `TASK.md` is the UTF-8 product brief. Keep it aligned with the code and
  human-facing documentation under `docs/`.
