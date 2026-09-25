import assert from 'node:assert/strict';
import test from 'node:test';

import { waitForFirstPage } from './windows-webview-smoke-utils.mjs';

test('waits for a WebView page exposed after the debugging endpoint', async () => {
  const expectedPage = { url: 'tauri://localhost' };
  let checks = 0;
  const browser = {
    contexts() {
      checks += 1;
      return [{ pages: () => (checks >= 3 ? [expectedPage] : []) }];
    },
  };

  const page = await waitForFirstPage(browser, { timeoutMs: 100, pollIntervalMs: 1 });

  assert.equal(page, expectedPage);
  assert.equal(checks, 3);
});
