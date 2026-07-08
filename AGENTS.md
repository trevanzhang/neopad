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
mcp-server/           Standalone MCP stdio server
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
- MCP must be read-only by default. Write behavior requires `--allow-write`.
- MCP stdout must contain only JSON-RPC protocol messages. Use stderr for logs.
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
- Keep the workspace version in `Cargo.toml`, the frontend version in
  `app/package.json`, and the Tauri version in `app/src-tauri/tauri.conf.json`
  synchronized. Update `Cargo.lock` when the workspace version changes.
- Do not bump versions for routine development commits. Bump the version when
  preparing a distinct distributable build or release.
- Use a patch bump for compatible fixes, a minor bump for backward-compatible
  features, and a major bump for incompatible changes.
- Use prerelease labels such as `beta` only when the task explicitly establishes
  a prerelease channel. Do not add them to ordinary development builds.

## Windows Packaging Notes

The current bundle target is MSI only. Do not re-enable NSIS unless the task is
specifically about NSIS packaging.

The installer uses a custom WiX template:

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
