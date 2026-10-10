import { createHash } from 'node:crypto'
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { basename, dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const target = join(root, 'target')
const { version, productName } = JSON.parse(
  readFileSync(join(root, 'app/src-tauri/tauri.conf.json'), 'utf8'),
)
const bundleRoot = process.platform === 'darwin'
  ? join(target, 'aarch64-apple-darwin', 'release', 'bundle')
  : join(target, 'release', 'bundle')
const suffixes = {
  win32: ['.msi'],
  darwin: ['.dmg'],
  linux: ['.deb', '.AppImage'],
}[process.platform]

if (!suffixes) {
  console.error(`Unsupported release platform: ${process.platform}`)
  process.exit(1)
}

function filesUnder(path) {
  if (!existsSync(path)) return []
  return readdirSync(path, { withFileTypes: true }).flatMap((entry) => {
    const child = join(path, entry.name)
    return entry.isDirectory() ? filesUnder(child) : [child]
  })
}

const artifacts = filesUnder(bundleRoot)
  .filter((path) => basename(path).startsWith(`${productName}_${version}_`))
  .filter((path) => suffixes.some((suffix) => path.endsWith(suffix)))
  .filter((path) => statSync(path).size > 0)
  .sort()

if (suffixes.some((suffix) => artifacts.filter((path) => path.endsWith(suffix)).length !== 1)) {
  console.error(`Expected one ${version} release artifact per format (${suffixes.join(', ')}), found ${artifacts.length} in ${bundleRoot}.`)
  process.exit(1)
}

const lines = artifacts.map((path) => {
  const digest = createHash('sha256').update(readFileSync(path)).digest('hex')
  return `${digest}  ${basename(path)}`
})
const output = join(target, `SHA256SUMS-${process.platform}.txt`)
writeFileSync(output, `${lines.join('\n')}\n`, 'utf8')
console.log(`Wrote ${output}`)
