import { readdir, readFile } from 'node:fs/promises'
import { extname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

const sourceRoot = fileURLToPath(new URL('../src/', import.meta.url))
const supportedExtensions = new Set(['.js', '.ts', '.vue'])
const deprecatedCustomProperty = /(?:[a-z-]+:)*[a-z][\w-]*-\[var\(--[^)\]]+\)\]/g
const findings = []

const scanDirectory = async (directory) => {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) {
      await scanDirectory(path)
      continue
    }
    if (!supportedExtensions.has(extname(entry.name))) continue

    const lines = (await readFile(path, 'utf8')).split(/\r?\n/)
    lines.forEach((line, index) => {
      for (const match of line.matchAll(deprecatedCustomProperty)) {
        findings.push(`${relative(sourceRoot, path)}:${index + 1}:${(match.index ?? 0) + 1} ${match[0]}`)
      }
    })
  }
}

await scanDirectory(sourceRoot)

if (findings.length > 0) {
  console.error('Use Tailwind 4 custom-property shorthand, for example text-(--color-text):')
  findings.forEach((finding) => console.error(`  ${finding}`))
  process.exitCode = 1
} else {
  console.log('Tailwind custom-property syntax is current.')
}
