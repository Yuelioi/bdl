import { describe, expect, it } from 'vitest'

import {
  archiveModeOptions,
  audioQualityOptions,
  codecOptions,
  concurrentTaskOptions,
  duplicateNamingOptions,
  logLevelOptions,
  missingQualityOptions,
  outputFormatOptions,
  retryCountOptions,
  segmentCountOptions,
  settingsSections,
  videoQualityOptions,
} from './settingsCatalog'

const optionCatalogs = [
  concurrentTaskOptions,
  retryCountOptions,
  videoQualityOptions,
  audioQualityOptions,
  outputFormatOptions,
  duplicateNamingOptions,
  codecOptions,
  missingQualityOptions,
  segmentCountOptions,
  archiveModeOptions,
  logLevelOptions,
]

describe('settings catalog', () => {
  it('keeps section ids unique', () => {
    const ids = settingsSections.map((section) => section.id)

    expect(new Set(ids).size).toBe(ids.length)
  })

  it('keeps option values unique within each setting', () => {
    for (const catalog of optionCatalogs) {
      const values = catalog.map((option) => option.value)
      expect(new Set(values).size).toBe(values.length)
    }
  })
})
