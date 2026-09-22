import { spawn, spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { chromium } from '@playwright/test';
import { resolveCargoTargetDir } from '../../../scripts/cargo-target.mjs';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, '..', '..', '..');
const executable = resolve(process.argv[2] ?? join(resolveCargoTargetDir(repoRoot), 'release', 'bdl-desktop.exe'));
const smokeConfigPath = resolve(scriptDir, '..', 'src-tauri', 'tauri.windows-smoke.conf.json');
const smokeConfig = JSON.parse(await readFile(smokeConfigPath, 'utf8'));
const browserArgs = smokeConfig.app?.windows?.[0]?.additionalBrowserArgs ?? '';
const portMatch = browserArgs.match(/(?:^|\s)--remote-debugging-port=(\d+)(?:\s|$)/);

if (!portMatch) {
  throw new Error(`Missing --remote-debugging-port in ${smokeConfigPath}`);
}

const port = Number(portMatch[1]);
if (!Number.isInteger(port) || port < 1 || port > 65_535) {
  throw new Error(`Invalid WebView2 debug port in ${smokeConfigPath}: ${portMatch[1]}`);
}

const assertPortAvailable = async () => {
  const server = createServer();
  await new Promise((resolveListen, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', resolveListen);
  });
  await new Promise((resolveClose, reject) => server.close((error) => (error ? reject(error) : resolveClose())));
};

await assertPortAvailable();
const endpoint = `http://127.0.0.1:${port}`;
const child = spawn(executable, [], {
  env: process.env,
  stdio: 'ignore',
});

let exit = null;
child.once('exit', (code, signal) => {
  exit = { code, signal };
});

const deadline = Date.now() + 20_000;
let browser;
let endpointReady = false;

try {
  while (Date.now() < deadline) {
    if (exit) throw new Error(`BDL exited before WebView2 became ready: ${JSON.stringify(exit)}`);
    try {
      const response = await fetch(`${endpoint}/json/version`);
      if (response.ok) {
        endpointReady = true;
        break;
      }
    } catch {
      // WebView2 has not opened its debugging endpoint yet.
    }
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 250));
  }

  if (!endpointReady) {
    if (exit) throw new Error(`BDL exited before WebView2 became ready: ${JSON.stringify(exit)}`);
    throw new Error(
      `WebView2 debugging endpoint did not become ready at ${endpoint}; build BDL with ${smokeConfigPath}`,
    );
  }

  browser = await chromium.connectOverCDP(endpoint, { timeout: 5_000 });
  const page = browser.contexts().flatMap((context) => context.pages())[0];
  if (!page) throw new Error('WebView2 started but no application page was exposed');

  const pageErrors = [];
  page.on('pageerror', (error) => pageErrors.push(error.message));

  await page.waitForFunction(
    () => {
      const root = document.querySelector('#app');
      return Boolean(root && root.children.length > 0 && document.body.innerText.trim().length > 20);
    },
    undefined,
    { timeout: 15_000 },
  );

  const rendered = await page.evaluate(() => ({
    readyState: document.readyState,
    rootChildren: document.querySelector('#app')?.children.length ?? 0,
    textLength: document.body.innerText.trim().length,
  }));

  if (pageErrors.length > 0) throw new Error(`Frontend page errors: ${pageErrors.join(' | ')}`);
  if (exit) throw new Error(`BDL exited during WebView smoke test: ${JSON.stringify(exit)}`);

  console.log(
    `Windows WebView smoke passed: readyState=${rendered.readyState}, rootChildren=${rendered.rootChildren}, textLength=${rendered.textLength}`,
  );
} finally {
  await browser?.close().catch(() => {});
  if (child.pid && !exit) {
    spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
  }
}
