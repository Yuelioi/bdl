import { readFileSync, readdirSync } from 'node:fs'
import { createRequire } from 'node:module'
import { extname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import vue from '@vitejs/plugin-vue'
import ui from '@nuxt/ui/vite'
import { defineConfig, type Plugin } from 'vite'

type IconData = {
  body: string
  width?: number
  height?: number
  left?: number
  top?: number
  rotate?: number
  hFlip?: boolean
  vFlip?: boolean
}

type IconAlias = Omit<IconData, 'body'> & { parent: string }

type IconCollection = {
  prefix: string
  width?: number
  height?: number
  icons: Record<string, IconData>
  aliases?: Record<string, IconAlias>
}

const virtualIconBundleId = 'virtual:bdl-icon-bundle'
const resolvedVirtualIconBundleId = `\0${virtualIconBundleId}`
const sourceDir = fileURLToPath(new URL('./src', import.meta.url))
const require = createRequire(import.meta.url)
const tablerCollectionPath = require.resolve('@iconify-json/tabler/icons.json')

const nuxtUiIcons = {
  arrowDown: 'i-tabler-arrow-down',
  arrowLeft: 'i-tabler-arrow-left',
  arrowRight: 'i-tabler-arrow-right',
  arrowUp: 'i-tabler-arrow-up',
  caution: 'i-tabler-alert-circle',
  check: 'i-tabler-check',
  chevronDoubleLeft: 'i-tabler-chevrons-left',
  chevronDoubleRight: 'i-tabler-chevrons-right',
  chevronDown: 'i-tabler-chevron-down',
  chevronLeft: 'i-tabler-chevron-left',
  chevronRight: 'i-tabler-chevron-right',
  chevronUp: 'i-tabler-chevron-up',
  close: 'i-tabler-x',
  copy: 'i-tabler-copy',
  copyCheck: 'i-tabler-copy-check',
  dark: 'i-tabler-moon',
  drag: 'i-tabler-grip-vertical',
  ellipsis: 'i-tabler-dots',
  error: 'i-tabler-circle-x',
  external: 'i-tabler-external-link',
  eye: 'i-tabler-eye',
  eyeOff: 'i-tabler-eye-off',
  file: 'i-tabler-file',
  folder: 'i-tabler-folder',
  folderOpen: 'i-tabler-folder-open',
  hash: 'i-tabler-hash',
  info: 'i-tabler-info-circle',
  light: 'i-tabler-sun',
  loading: 'i-tabler-loader-2',
  menu: 'i-tabler-menu-2',
  minus: 'i-tabler-minus',
  panelClose: 'i-tabler-layout-sidebar-left-collapse',
  panelOpen: 'i-tabler-layout-sidebar-left-expand',
  plus: 'i-tabler-plus',
  reload: 'i-tabler-refresh',
  search: 'i-tabler-search',
  stop: 'i-tabler-square',
  success: 'i-tabler-circle-check',
  system: 'i-tabler-device-desktop',
  tip: 'i-tabler-bulb',
  upload: 'i-tabler-upload',
  warning: 'i-tabler-alert-triangle',
} as const

const collectSourceFiles = (directory: string): string[] =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return collectSourceFiles(path)
    return extname(entry.name) === '.ts' || extname(entry.name) === '.vue' ? [path] : []
  })

const buildLocalIconCollection = (sourceFiles: string[]): IconCollection => {
  const collection = JSON.parse(readFileSync(tablerCollectionPath, 'utf8')) as IconCollection
  const validNames = new Set([...Object.keys(collection.icons), ...Object.keys(collection.aliases ?? {})])
  const selectedNames = new Set<string>()

  for (const icon of Object.values(nuxtUiIcons)) selectedNames.add(icon.replace(/^i-tabler-/, ''))

  for (const file of sourceFiles) {
    const source = readFileSync(file, 'utf8')

    for (const match of source.matchAll(/\bi-tabler-([a-z0-9-]+)\b/g)) selectedNames.add(match[1])

    for (const match of source.matchAll(/(['"`])([a-z0-9][a-z0-9-]*)\1/g)) {
      if (validNames.has(match[2])) selectedNames.add(match[2])
    }
  }

  const icons: Record<string, IconData> = {}
  const aliases: Record<string, IconAlias> = {}
  const added = new Set<string>()

  const addIcon = (name: string) => {
    if (added.has(name)) return
    added.add(name)

    const icon = collection.icons[name]
    if (icon) {
      icons[name] = icon
      return
    }

    const alias = collection.aliases?.[name]
    if (alias) {
      aliases[name] = alias
      addIcon(alias.parent)
    }
  }

  for (const name of selectedNames) addIcon(name)

  return {
    prefix: collection.prefix,
    width: collection.width,
    height: collection.height,
    icons,
    ...(Object.keys(aliases).length > 0 ? { aliases } : {}),
  }
}

const localIconBundle = (): Plugin => ({
  name: 'bdl:local-icon-bundle',
  resolveId(id) {
    if (id === virtualIconBundleId) return resolvedVirtualIconBundleId
  },
  load(id) {
    if (id !== resolvedVirtualIconBundleId) return
    const sourceFiles = collectSourceFiles(sourceDir)
    for (const file of sourceFiles) this.addWatchFile(file)
    return `export default ${JSON.stringify(buildLocalIconCollection(sourceFiles))}`
  },
})

export default defineConfig({
  plugins: [
    localIconBundle(),
    vue(),
    ui({
      ui: {
        icons: nuxtUiIcons,
      },
      theme: {
        colors: ['primary', 'success', 'warning', 'error', 'neutral'],
        defaultVariants: {
          color: 'primary',
          size: 'sm',
        },
      },
    }),
  ],
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
  },
})
