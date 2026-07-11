import { createHash } from 'node:crypto'
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const target = join(root, 'target')
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

const artifacts = filesUnder(target)
  .filter((path) => suffixes.some((suffix) => path.endsWith(suffix)))
  .filter((path) => statSync(path).size > 0)
  .sort()

if (artifacts.length !== suffixes.length) {
  console.error(`Expected ${suffixes.length} release artifacts, found ${artifacts.length}.`)
  process.exit(1)
}

const lines = artifacts.map((path) => {
  const digest = createHash('sha256').update(readFileSync(path)).digest('hex')
  return `${digest}  ${relative(root, path).replaceAll('\\', '/')}`
})
const output = join(target, `SHA256SUMS-${process.platform}.txt`)
writeFileSync(output, `${lines.join('\n')}\n`, 'utf8')
console.log(`Wrote ${output}`)
