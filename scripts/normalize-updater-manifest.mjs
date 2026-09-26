import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

function assetUrls(asset, tag) {
  const apiUrl = asset.apiUrl ?? asset.url
  const downloadUrl = asset.browser_download_url ?? asset.url
  if (tag) {
    const match = /^https:\/\/api\.github\.com\/repos\/([^/]+\/[^/]+)\/releases\/assets\/\d+$/.exec(apiUrl)
    if (!match || !asset.name) {
      throw new Error(`Release asset ${asset.name ?? '(unnamed)'} has no GitHub API URL.`)
    }
    return {
      apiUrl,
      downloadUrl: `https://github.com/${match[1]}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(asset.name)}`,
      stagedUrl: downloadUrl,
    }
  }
  if (asset.apiUrl) {
    return {
      apiUrl: asset.apiUrl,
      downloadUrl: asset.url,
    }
  }

  return {
    apiUrl: asset.url,
    downloadUrl: asset.browser_download_url,
  }
}

export function normalizeUpdaterManifest(manifest, assets, tag) {
  const normalized = structuredClone(manifest)
  let updatedEntries = 0

  for (const [target, entry] of Object.entries(normalized.platforms ?? {})) {
    if (!entry?.url) {
      throw new Error(`Updater target ${target} is missing its download URL.`)
    }

    const asset = assets.find((candidate) => {
      const { apiUrl, downloadUrl, stagedUrl } = assetUrls(candidate, tag)
      return entry.url === apiUrl || entry.url === downloadUrl || entry.url === stagedUrl
    })
    if (!asset) {
      throw new Error(`Updater target ${target} does not match a release asset: ${entry.url}`)
    }

    const { downloadUrl } = assetUrls(asset, tag)
    if (!downloadUrl?.startsWith('https://github.com/')) {
      throw new Error(`Release asset for ${target} has no public GitHub download URL.`)
    }
    if (entry.url !== downloadUrl) {
      entry.url = downloadUrl
      updatedEntries += 1
    }
  }

  return { manifest: normalized, updatedEntries }
}

function main() {
  const [, , manifestPath, assetsPath, tag, outputPath = manifestPath] = process.argv
  if (!manifestPath || !assetsPath) {
    throw new Error(
      'Usage: node scripts/normalize-updater-manifest.mjs <manifest.json> <assets.json> <tag> [output.json]',
    )
  }

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
  const assetsDocument = JSON.parse(fs.readFileSync(assetsPath, 'utf8'))
  const assets = Array.isArray(assetsDocument) ? assetsDocument : assetsDocument.assets
  if (!Array.isArray(assets)) {
    throw new Error('Release assets input must be an array or an object containing an assets array.')
  }
  if (!/^v\d+\.\d+\.\d+$/.test(tag ?? '')) {
    throw new Error('A release tag such as v0.6.1 is required.')
  }
  const result = normalizeUpdaterManifest(manifest, assets, tag)

  fs.mkdirSync(path.dirname(outputPath), { recursive: true })
  fs.writeFileSync(outputPath, `${JSON.stringify(result.manifest, null, 2)}\n`)
  process.stdout.write(`Normalized ${result.updatedEntries} updater URL(s).\n`)
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main()
}
