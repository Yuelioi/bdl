import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const requested = process.argv[2]
const manifestPaths = [
  'crates/bdl-core/Cargo.toml',
  'crates/bdl-tauri/Cargo.toml',
  'crates/bdl-cli/Cargo.toml',
  'apps/desktop/src-tauri/Cargo.toml',
]
const packagePath = 'apps/desktop/package.json'
const tauriConfigPath = 'apps/desktop/src-tauri/tauri.conf.json'

if (!requested) {
  throw new Error('Usage: ./scripts/version.sh <patch|minor|major|x.y.z>')
}

const read = (path) => readFileSync(resolve(repoRoot, path), 'utf8')
const write = (path, content) => writeFileSync(resolve(repoRoot, path), content, 'utf8')
const manifestVersion = (path) => {
  const match = read(path).match(/^version = "(\d+\.\d+\.\d+)"/m)
  if (!match) throw new Error(`No package version found in ${path}.`)
  return match[1]
}

const versions = [
  ...manifestPaths.map(manifestVersion),
  JSON.parse(read(packagePath)).version,
  JSON.parse(read(tauriConfigPath)).version,
]
const current = versions[0]
if (versions.some((version) => version !== current)) {
  throw new Error(`Project versions are inconsistent: ${versions.join(', ')}. Fix them before bumping.`)
}

const [major, minor, patch] = current.split('.').map(Number)
const next =
  requested === 'major'
    ? `${major + 1}.0.0`
    : requested === 'minor'
      ? `${major}.${minor + 1}.0`
      : requested === 'patch'
        ? `${major}.${minor}.${patch + 1}`
        : /^\d+\.\d+\.\d+$/.test(requested)
          ? requested
          : null

if (!next) throw new Error('Version must be patch, minor, major, or an exact x.y.z version.')
if (next === current) throw new Error(`The requested version is already ${current}.`)

for (const path of manifestPaths) {
  const content = read(path)
  const updated = content.replace(/^version = "\d+\.\d+\.\d+"/m, `version = "${next}"`)
  if (updated === content) throw new Error(`Failed to update ${path}.`)
  write(path, updated)
}

for (const path of [packagePath, tauriConfigPath]) {
  const content = read(path)
  const updated = content.replace(/^(\s*"version"\s*:\s*)"\d+\.\d+\.\d+"/m, `$1"${next}"`)
  if (updated === content) throw new Error(`Failed to update ${path}.`)
  write(path, updated)
}

write('README.md', read('README.md').replace(/当前版本：`\d+\.\d+\.\d+`/, `当前版本：\`${next}\``))

let lock = read('Cargo.lock')
for (const packageName of ['bdl-core', 'bdl-tauri', 'bdl-cli', 'bdl-desktop']) {
  const packagePattern = new RegExp(
    `(\\[\\[package\\]\\]\\r?\\nname = "${packageName}"\\r?\\nversion = ")\\d+\\.\\d+\\.\\d+(")`,
  )
  const updated = lock.replace(packagePattern, `$1${next}$2`)
  if (updated === lock) throw new Error(`Failed to update ${packageName} in Cargo.lock.`)
  lock = updated
}
write('Cargo.lock', lock)

console.log(`Version updated: ${current} -> ${next}`)
console.log('Run ./scripts/check.sh before creating the release tag.')
