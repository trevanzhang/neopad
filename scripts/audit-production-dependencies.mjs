import { execFileSync } from 'node:child_process'

const pnpmEntryPoint = process.env.npm_execpath
if (!pnpmEntryPoint) throw new Error('Run this audit through the pnpm audit:prod script.')

const workspace = JSON.parse(execFileSync(
  process.execPath,
  [pnpmEntryPoint, 'list', '-r', '--prod', '--json', '--depth', 'Infinity'],
  {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  },
))

const versionsByPackage = new Map()

function collectDependencies(dependencies = {}) {
  for (const [name, dependency] of Object.entries(dependencies)) {
    if (!dependency?.version) continue
    const versions = versionsByPackage.get(name) ?? new Set()
    versions.add(dependency.version)
    versionsByPackage.set(name, versions)
    collectDependencies(dependency.dependencies)
  }
}

for (const project of workspace) collectDependencies(project.dependencies)

const payload = Object.fromEntries(
  [...versionsByPackage.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([name, versions]) => [name, [...versions].sort()]),
)

const response = await fetch('https://registry.npmjs.org/-/npm/v1/security/advisories/bulk', {
  method: 'POST',
  headers: {
    accept: 'application/json',
    'content-type': 'application/json',
  },
  body: JSON.stringify(payload),
})

if (!response.ok) {
  const details = await response.text()
  throw new Error(`npm Bulk Advisory API returned ${response.status}: ${details}`)
}

const advisories = await response.json()
const findings = Object.entries(advisories)
  .flatMap(([name, entries]) => entries.map((entry) => ({ name, ...entry })))
  .sort((left, right) => left.name.localeCompare(right.name))

if (findings.length > 0) {
  console.error(`Found ${findings.length} production dependency advisories:`)
  for (const finding of findings) {
    console.error(
      `- ${finding.name} ${finding.vulnerable_versions}: ${finding.severity} ${finding.title} (${finding.url})`,
    )
  }
  process.exit(1)
}

const versionCount = [...versionsByPackage.values()]
  .reduce((total, versions) => total + versions.size, 0)
console.log(`Audited ${versionCount} exact production dependency versions; no advisories found.`)
