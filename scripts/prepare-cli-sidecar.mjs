// Cross-platform CLI sidecar preparation.
//
// Copies the release neopad-cli binary to the target-triple-suffixed name
// that Tauri's externalBin convention expects:
//
//   neopad-cli-x86_64-pc-windows-msvc.exe   (Windows)
//   neopad-cli-aarch64-apple-darwin          (macOS ARM64)
//   neopad-cli-x86_64-unknown-linux-gnu      (Linux x64)
//
// Used by platform builds and the bundled native CLI.

import { execFileSync } from 'node:child_process'
import { copyFileSync, existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = dirname(fileURLToPath(import.meta.url))
const repoRoot = dirname(scriptDir)

// Determine the Rust target triple. When cross-compiling (e.g. in CI with
// --target), prefer the target triple over the host triple so the suffix
// matches what Tauri looks for.
function getTargetTriple() {
  const cargoTarget = process.env.CARGO_BUILD_TARGET
  if (cargoTarget) return cargoTarget.trim()

  const rustcTarget = process.env.RUSTC_TARGET
  if (rustcTarget) return rustcTarget.trim()

  // Fall back to the host triple
  const output = execFileSync('rustc', ['-vV'], { encoding: 'utf-8' })
  const hostLine = output
    .split('\n')
    .find((line) => line.startsWith('host:'))
  if (!hostLine) {
    console.error('Could not determine Rust host target from `rustc -vV`.')
    process.exit(1)
  }
  return hostLine.slice('host:'.length).trim()
}

const triple = getTargetTriple()
const exeSuffix = triple.includes('windows') ? '.exe' : ''
const explicitTarget = Boolean(process.env.CARGO_BUILD_TARGET?.trim() || process.env.RUSTC_TARGET?.trim())

// The CLI binary may be in target/<triple>/release/ (cross-compiled) or
// target/release/ (native build). Do not silently label a native binary as a
// different architecture if an explicitly selected target build is missing.
const crossDir = join(repoRoot, 'target', triple, 'release')
const nativeDir = join(repoRoot, 'target', 'release')
const source = join(explicitTarget ? crossDir : nativeDir, `neopad-cli${exeSuffix}`)

if (!existsSync(source)) {
  console.error(
    `Missing CLI release binary: ${source}. Run \`cargo build -p neopad-cli --release${explicitTarget ? ` --target ${triple}` : ''}\` first.`
  )
  process.exit(1)
}

// Tauri's externalBin path in tauri.conf.json is `../../target/release/neopad-cli`,
// so the suffixed sidecar must always be placed in target/release/ — even for
// cross-compiled builds where the binary was produced under target/<triple>/release/.
const dest = join(nativeDir, `neopad-cli-${triple}${exeSuffix}`)

copyFileSync(source, dest)
console.log(`Prepared CLI sidecar: ${dest}`)
