export function firstPage(browser) {
  return browser.contexts().flatMap((context) => context.pages())[0];
}

export async function waitForFirstPage(
  browser,
  { timeoutMs = 15_000, pollIntervalMs = 100 } = {},
) {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() <= deadline) {
    const page = firstPage(browser);
    if (page) return page;
    await new Promise((resolveDelay) => setTimeout(resolveDelay, pollIntervalMs));
  }

  throw new Error(`WebView2 started but exposed no application page within ${timeoutMs}ms`);
}
