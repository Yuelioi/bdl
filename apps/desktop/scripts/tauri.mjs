import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { resolveCargoTargetDir } from '../../../scripts/cargo-target.mjs'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const desktopDir = path.resolve(scriptDir, '..')
const repoRoot = path.resolve(desktopDir, '..', '..')
process.env.CARGO_TARGET_DIR = resolveCargoTargetDir(repoRoot)

await import('@tauri-apps/cli/tauri.js')
