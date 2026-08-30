import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const version = JSON.parse(readFileSync(resolve(repoRoot, 'apps/desktop/package.json'), 'utf8')).version
const tag = process.argv[2] ?? process.env.GITHUB_REF_NAME
const expected = `v${version}`

if (tag !== expected) {
  throw new Error(`Release tag '${tag ?? ''}' does not match app version '${expected}'.`)
}

console.log(`Release tag verified: ${tag}`)
