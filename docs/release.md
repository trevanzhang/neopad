# Release Preparation

NeoPad builds local installers separately from GitHub Releases. A green CI run
validates source code; it does not upload installers. The `Release` workflow
starts only after a matching `v*` tag is pushed and uploads to a draft Release.

## Version and Upgrade Notes

Prepare each distinct distributable build with a synchronized version in
`Cargo.toml`, `package.json`, `app/package.json`, `app/src-tauri/tauri.conf.json`,
and the NeoPad packages in `Cargo.lock`. Use a patch for compatible fixes and
record the changes under a dated version section in `CHANGELOG.md`.

The current preparation target is **0.7.2**; see
[release notes](releases/v0.7.2.md). Before upgrading from 0.6.x or earlier,
back up the entire workspace, including metadata, legacy prompts, and Trash.
MCP connections must [migrate to the bundled CLI](cli.md#migrate-from-mcp), and
legacy prompts [migrate automatically to project notes](projects.md).
Launch the upgraded desktop once to deploy the matching CLI. Existing
workspace-root `AGENTS.md` files are preserved, including empty guides.

## Local Windows Build

Run from the repository root on a dedicated branch. Preserve uncommitted user
files. The normal build does not install the app or run workspace migrations.

```powershell
pnpm install --frozen-lockfile
pnpm check:versions
pnpm check:release-tag v0.7.2
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm test:all
pnpm audit:prod
cargo audit
pnpm tauri:build
node scripts/write-release-checksums.mjs
```

The root build includes the release CLI, its target-specific sidecar, the
frontend, and the desktop bundle. Windows output:

```text
target/release/bundle/msi/NeoPad_0.7.2_x64_en-US.msi
target/release/neopad-app.exe
target/release/neopad-cli.exe
target/SHA256SUMS-win32.txt
```

The checksum script selects the current Tauri version from the platform bundle
directory and requires exactly one nonempty artifact per expected format.
Older versions elsewhere in `target/` are preserved and excluded. Manifests use
installer filenames so downloaded assets can be verified together in one folder.

Compare the manifest with the installer hash and inspect its signature:

```powershell
Get-Content target/SHA256SUMS-win32.txt
Get-FileHash target/release/bundle/msi/NeoPad_0.7.2_x64_en-US.msi -Algorithm SHA256
Get-AuthenticodeSignature target/release/bundle/msi/NeoPad_0.7.2_x64_en-US.msi
```

Run the [packaged Windows smoke checklist](release-smoke-checklist.md) in a
disposable test environment before creating a Release. Desktop E2E uses its
own workspace (`pnpm test:e2e`); it does not substitute for MSI install/upgrade
checks. Do not install into a user's live workspace merely to validate a build.

## GitHub Draft Build

Commit, push, and tag only when requested by the repository owner. Start with
the reviewed preparation branch, fast-forward `develop`, and push it. Wait for
all CI jobs on that commit, including Windows desktop E2E and macOS/Linux
checks, to pass. Then fast-forward `main` to the same commit and verify its CI.
Use the release version's tag on `main`:

```text
git tag v0.7.2
git push origin v0.7.2
```

Do not reuse a published tag. The workflow checks versions, the tag, production
dependencies, formatting, Clippy, tests, and frontend compilation before
building the following matrix:

| Platform | Installer | Checksum manifest |
| --- | --- | --- |
| Windows x64 | `.msi` | `SHA256SUMS-win32.txt` |
| macOS ARM64 | `.dmg` | `SHA256SUMS-darwin.txt` |
| Linux x64 | `.deb`, `.AppImage` | `SHA256SUMS-linux.txt` |

All four installers and three manifests must be present in the draft. Use
`docs/releases/v0.7.2.md` as the release body after updating its status and
validation results. A successful build must remain a draft until publication
requirements have been satisfied.

## Publication Requirements

The current workflow builds bundles and hashes but has no platform signing or
notarization configuration. A local unsigned MSI is suitable for private build
validation; it is not a verified public release. Do not publish unsigned public
installers.

Before publishing, verify Windows Authenticode and macOS signing/notarization
with project-owned credentials. Verify the executable and bundled CLI as well
as the installers, and perform the platform smoke checks. Verify Linux artifacts
and manifests against the project's release signing policy. Sign artifacts
before generating their final hashes; regenerate and verify manifests if
signing changes any uploaded file. Set the release title and notes only after
recording the actual build, smoke, and signature results.

Known runtime limitations: window opacity and autostart remain Windows-only;
macOS and Linux compile those settings as no-ops.
