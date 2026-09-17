import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

export const resolveCargoTargetDir = (repoRoot, configuredTargetRoot = process.env.CARGO_TARGET_DIR) => {
  const resolvedRepoRoot = path.resolve(repoRoot)
  const projectName = path.basename(resolvedRepoRoot)
  const configured = configuredTargetRoot?.trim()

  if (!configured) return path.join(resolvedRepoRoot, 'target')

  const normalizedTargetRoot = path.resolve(configured)
  if (path.basename(normalizedTargetRoot).toLowerCase() === projectName.toLowerCase()) {
    return normalizedTargetRoot
  }

  return path.join(normalizedTargetRoot, projectName)
}

const isDirectRun = process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href
if (isDirectRun) {
  const repoRoot = process.argv[2] ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
  process.stdout.write(resolveCargoTargetDir(repoRoot))
}
