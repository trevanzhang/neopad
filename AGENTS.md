# AGENTS.md

This guide is for coding agents working in this repository.

## Project Summary

NeoPad is a local-first desktop note pad built with Tauri 2, Vue 3,
TypeScript, and Rust. It stores user notes as Markdown files and exposes the
same local data through a native CLI.

Keep NeoPad small and fast. It is a capture tool, not a knowledge-base suite.

## Repository Layout

```text
app/                  Vue frontend and Tauri desktop app
app/src/              Vue components, styles, TypeScript helpers
app/src-tauri/        Tauri Rust shell, commands, tray, hotkeys, packaging
crates/neopad-core/   Shared Rust core for workspace and note operations
cli/                  Standalone native CLI
docs/                 Human-facing project docs
```

## Important Rules

- Use `neopad-core` for all note, workspace, path, search, and write behavior.
- Do not duplicate filesystem access logic in the Tauri shell or CLI.
- Capture note content stays in flat `~/.neopad/notes/*.md`. Active project
  notes live under `~/.neopad/projects/**/*.md` and use ordinary note APIs.
  Archived notes may be categorized under `~/.neopad/archive/**/*.md`.
- Metadata belongs in `config.json`, `meta/tabs.json`, and
  `meta/reminders.json`. The reminder file stores delivery state only; reminder
  content remains in Markdown notes.
- The reusable prompt library is retired. Preserve legacy prompt Markdown and
  trash records when migrating them to project notes; keep migration resumable.
- Never physically delete user notes. Move notes to `trash/`.
- Keep path safety strict. Reject absolute paths, `..`, and file names that
  escape the workspace.
- Deploy the bundled CLI to `.agents/bin/` through `neopad-core`.
- Never discover or launch a replacement CLI from the system PATH.
- Preserve user-authored files under `.agents/`; only manage the CLI binary.
- CLI results must be JSON; full replacements require an expected timestamp.
- CLI note operations must share the desktop workspace write lock.
- Do not introduce cloud sync, accounts, RAG, vector search, backlinks,
  persistent AI chat history, or an AI chat sidebar unless the task explicitly
  changes scope. Keep inline AI user-initiated and reviewable.

## Build Commands

Run these from the repository root.

```powershell
cargo test
pnpm build
pnpm tauri:build
```

For CLI changes:

```powershell
cargo build -p neopad-cli --release
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
- During initial 0.x development, incompatible Agent interface changes may use
  the next minor version when directed by the user. Document these changes and
  migration steps explicitly; do not advance to 1.0.0 solely for such a change.
- Use prerelease labels such as `beta` only when the task explicitly establishes
  a prerelease channel. Do not add them to ordinary development builds.

## Release Process

Releases are built automatically by GitHub Actions when a `v*` tag is pushed.

1. Ensure `main` and `develop` are in sync and all CI checks pass.
2. Bump versions (see Versioning above), commit to `develop`, and fast-forward
   merge into `main`.
3. Tag the release on `main` and push:

   ```text
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

4. The `Release` workflow builds all three platforms in parallel and uploads
   installers to the GitHub Release for that tag:

   ```text
   Windows x64   .msi     (windows-latest)
   macOS ARM64   .dmg     (macos-latest, --target aarch64-apple-darwin)
   Linux x64     .deb     (ubuntu-22.04)
   Linux x64     .AppImage (ubuntu-22.04)
   ```

5. The workflow creates a draft Release. Before publishing it, verify the
   SHA-256 manifests and platform signatures/notarization, then set the title
   and notes. Never publish unsigned public installers.
6. Update `CHANGELOG.md`: move entries from `## Unreleased` into a dated
   `## x.y.z` section if not already done.

Do not delete or recreate published tags. If a release needs rebuilding, delete
the GitHub Release and re-push the tag.

## Cross-Platform Packaging Notes

The build scripts select an explicit bundle set per platform:

- Windows: `.msi` (WiX)
- macOS: `.dmg`
- Linux: `.deb` and `.AppImage`

`tauri.conf.json` defaults to MSI for direct Windows builds, while the
cross-platform scripts override that default with the platform-specific set.
Do not use `targets: "all"` or re-enable NSIS unless the task is specifically
about NSIS packaging.

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

### CLI Sidecar

The native CLI is bundled as a Tauri `externalBin` sidecar.
`scripts/prepare-cli-sidecar.mjs` prepares the target-triple-suffixed binary.
`tauri:build` runs this automatically. The desktop deploys the trusted bundled
binary to `~/.neopad/.agents/bin/` at startup and can repair it from Settings.

## Frontend Guidelines

- Keep the UI compact and utility-focused.
- Keep the main window `dragDropEnabled` setting disabled. NeoPad's tab and
  library reordering use HTML drag events, which Tauri's native Windows file
  drop handling intercepts when enabled.
- Do not add a landing page.
- Prefer existing component and CSS patterns.
- Keep text within fixed UI controls from overflowing.
- Use the existing logo assets from `app/src/assets/`.
- Avoid decorative-only redesigns unless the task is specifically visual.

## Core Data Guidelines

- Prefer focused core functions with tests over UI-side workarounds.
- Preserve atomic write behavior.
- Preserve `expectedUpdatedAt` conflict checks for full-page CLI updates.
- Pinned default pages such as Inbox and Clipboard must not be deletable.

## Documentation Guidelines

- Keep Markdown files UTF-8.
- Prefer ASCII unless non-ASCII content is required.
- Update `README.md` and `docs/` when commands, packaging, or user-visible
  behavior changes.
- `TASK.md` is the UTF-8 product brief. Keep it aligned with the code and
  human-facing documentation under `docs/`.
