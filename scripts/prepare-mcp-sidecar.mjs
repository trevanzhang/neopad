// Cross-platform MCP sidecar preparation.
//
// Copies the release neopad-mcp binary to the target-triple-suffixed name
// that Tauri's externalBin convention expects:
//
//   neopad-mcp-x86_64-pc-windows-msvc.exe   (Windows)
//   neopad-mcp-aarch64-apple-darwin          (macOS ARM64)
//   neopad-mcp-x86_64-unknown-linux-gnu      (Linux x64)
//
// Replaces the Windows-only prepare-mcp-sidecar.ps1 so the same
// `pnpm tauri:build` command works on every platform.

import { execFileSync } from 'node:child_process'
import { copyFileSync, existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = dirname(fileURLToPath(import.meta.url))
const repoRoot = dirname(scriptDir)

const isWindows = process.platform === 'win32'
const exeSuffix = isWindows ? '.exe' : ''

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

// The MCP binary may be in target/<triple>/release/ (cross-compiled) or
// target/release/ (native build). Find it wherever it was produced.
const crossDir = join(repoRoot, 'target', triple, 'release')
const nativeDir = join(repoRoot, 'target', 'release')
const source = existsSync(join(crossDir, `neopad-mcp${exeSuffix}`))
  ? join(crossDir, `neopad-mcp${exeSuffix}`)
  : join(nativeDir, `neopad-mcp${exeSuffix}`)

if (!existsSync(source)) {
  console.error(
    `Missing MCP release binary: ${source}. Run \`cargo build -p neopad-mcp --release\` first.`
  )
  process.exit(1)
}

// Tauri's externalBin path in tauri.conf.json is `../../target/release/neopad-mcp`,
// so the suffixed sidecar must always be placed in target/release/ — even for
// cross-compiled builds where the binary was produced under target/<triple>/release/.
const dest = join(nativeDir, `neopad-mcp-${triple}${exeSuffix}`)

copyFileSync(source, dest)
console.log(`Prepared MCP sidecar: ${dest}`)
