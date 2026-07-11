import { spawnSync } from 'node:child_process'

const bundlesByPlatform = {
  win32: 'msi',
  darwin: 'dmg',
  linux: 'deb,appimage',
}

const bundles = bundlesByPlatform[process.platform]
if (!bundles) {
  console.error(`Unsupported bundle platform: ${process.platform}`)
  process.exit(1)
}

const command = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'
const result = spawnSync(
  command,
  ['--filter', 'neopad-app', 'tauri', 'build', '--bundles', bundles],
  { stdio: 'inherit', shell: process.platform === 'win32' },
)

if (result.error) {
  console.error(result.error.message)
  process.exit(1)
}
process.exit(result.status ?? 1)
