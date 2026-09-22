import { spawn, spawnSync } from 'node:child_process'
import { mkdtemp, rm } from 'node:fs/promises'
import { createServer } from 'node:net'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { chromium } from '@playwright/test'
import { resolveCargoTargetDir } from '../../../scripts/cargo-target.mjs'

const scriptDir = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(scriptDir, '..', '..', '..')
const executable = resolve(
  process.argv[2] ?? join(resolveCargoTargetDir(repoRoot), 'release', 'bdl-desktop.exe'),
)
const userDataDir = await mkdtemp(join(tmpdir(), 'bdl-webview-smoke-'))

const freePort = async () => {
  const server = createServer()
  await new Promise((resolveListen, reject) => {
    server.once('error', reject)
    server.listen(0, '127.0.0.1', resolveListen)
  })
  const address = server.address()
  if (!address || typeof address === 'string') throw new Error('Unable to allocate a WebView2 debug port')
  await new Promise((resolveClose, reject) => server.close((error) => (error ? reject(error) : resolveClose())))
  return address.port
}

const port = await freePort()
const endpoint = `http://127.0.0.1:${port}`
const child = spawn(executable, [], {
  env: {
    ...process.env,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    WEBVIEW2_USER_DATA_FOLDER: userDataDir,
  },
  stdio: 'ignore',
})

let exit = null
child.once('exit', (code, signal) => {
  exit = { code, signal }
})

const deadline = Date.now() + 20_000
let browser

try {
  while (Date.now() < deadline) {
    if (exit) throw new Error(`BDL exited before WebView2 became ready: ${JSON.stringify(exit)}`)
    try {
      const response = await fetch(`${endpoint}/json/version`)
      if (response.ok) break
    } catch {
      // WebView2 has not opened its debugging endpoint yet.
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 250))
  }

  browser = await chromium.connectOverCDP(endpoint, { timeout: 5_000 })
  const page = browser.contexts().flatMap((context) => context.pages())[0]
  if (!page) throw new Error('WebView2 started but no application page was exposed')

  const pageErrors = []
  page.on('pageerror', (error) => pageErrors.push(error.message))

  await page.waitForFunction(
    () => {
      const root = document.querySelector('#app')
      return Boolean(root && root.children.length > 0 && document.body.innerText.trim().length > 20)
    },
    undefined,
    { timeout: 15_000 },
  )

  const rendered = await page.evaluate(() => ({
    readyState: document.readyState,
    rootChildren: document.querySelector('#app')?.children.length ?? 0,
    textLength: document.body.innerText.trim().length,
  }))

  if (pageErrors.length > 0) throw new Error(`Frontend page errors: ${pageErrors.join(' | ')}`)
  if (exit) throw new Error(`BDL exited during WebView smoke test: ${JSON.stringify(exit)}`)

  console.log(
    `Windows WebView smoke passed: readyState=${rendered.readyState}, rootChildren=${rendered.rootChildren}, textLength=${rendered.textLength}`,
  )
} finally {
  await browser?.close().catch(() => {})
  if (child.pid && !exit) {
    spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' })
  }
  await rm(userDataDir, { recursive: true, force: true }).catch(() => {})
}
