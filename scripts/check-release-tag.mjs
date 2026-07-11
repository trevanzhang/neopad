import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const version = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version
const tag = process.env.GITHUB_REF_NAME ?? process.argv[2]

if (!tag) {
  console.error('Release tag was not provided.')
  process.exit(1)
}
if (tag !== `v${version}`) {
  console.error(`Release tag ${tag} does not match package version v${version}.`)
  process.exit(1)
}
console.log(`Release tag ${tag} matches NeoPad version ${version}.`)
