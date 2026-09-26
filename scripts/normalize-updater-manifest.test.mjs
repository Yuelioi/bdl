import assert from 'node:assert/strict'
import test from 'node:test'

import { normalizeUpdaterManifest } from './normalize-updater-manifest.mjs'

const assets = [
  {
    name: 'BDL_0.6.0_x64_en-US.msi',
    url: 'https://api.github.com/repos/Yuelioi/bdl/releases/assets/1',
    browser_download_url:
      'https://github.com/Yuelioi/bdl/releases/download/v0.6.0/BDL_0.6.0_x64_en-US.msi',
  },
  {
    name: 'BDL_0.6.0_amd64.AppImage',
    url: 'https://api.github.com/repos/Yuelioi/bdl/releases/assets/2',
    browser_download_url:
      'https://github.com/Yuelioi/bdl/releases/download/v0.6.0/BDL_0.6.0_amd64.AppImage',
  },
]

test('replaces asset API URLs and preserves signatures', () => {
  const signature = 'signed-update'
  const result = normalizeUpdaterManifest(
    {
      version: '0.6.0',
      platforms: {
        'windows-x86_64': {
          signature,
          url: assets[0].url,
        },
        'linux-x86_64': {
          signature,
          url: assets[1].browser_download_url,
        },
      },
    },
    assets,
  )

  assert.equal(result.updatedEntries, 1)
  assert.equal(result.manifest.platforms['windows-x86_64'].url, assets[0].browser_download_url)
  assert.equal(result.manifest.platforms['windows-x86_64'].signature, signature)
  assert.equal(result.manifest.platforms['linux-x86_64'].url, assets[1].browser_download_url)
})

test('rejects updater targets that do not match an uploaded asset', () => {
  assert.throws(
    () =>
      normalizeUpdaterManifest(
        {
          platforms: {
            'windows-x86_64': {
              signature: 'signed-update',
              url: 'https://api.github.com/repos/Yuelioi/bdl/releases/assets/missing',
            },
          },
        },
        assets,
      ),
    /does not match a release asset/,
  )
})

test('uses the tagged download URL for a draft release asset', () => {
  const draftAsset = {
    ...assets[1],
    browser_download_url:
      'https://github.com/Yuelioi/bdl/releases/download/untagged-abc/BDL_0.6.0_amd64.AppImage',
  }
  const result = normalizeUpdaterManifest(
    { platforms: { 'linux-x86_64-appimage': { signature: 'signed-update', url: draftAsset.url } } },
    [draftAsset],
    'v0.6.0',
  )
  assert.equal(result.manifest.platforms['linux-x86_64-appimage'].url, assets[1].browser_download_url)
})
