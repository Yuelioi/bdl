<template>
  <main class="app-shell">
    <header class="app-header">
      <span class="brand-mark">BDL</span>
    </header>

    <section class="workspace" aria-labelledby="app-title">
      <h1 id="app-title">BDL</h1>
    </section>
  </main>
</template>

<style scoped>
:global(*) {
  box-sizing: border-box;
}

:global(body) {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
  color: #1f2937;
  background: #f6f7f9;
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
    sans-serif;
}

.app-shell {
  min-height: 100vh;
  display: grid;
  grid-template-rows: auto 1fr;
}

.app-header {
  height: 56px;
  display: flex;
  align-items: center;
  padding: 0 24px;
  border-bottom: 1px solid #d9dee7;
  background: #ffffff;
}

.brand-mark {
  font-size: 14px;
  font-weight: 700;
  color: #111827;
}

.workspace {
  display: grid;
  place-items: center;
  padding: 32px;
}

.workspace h1 {
  margin: 0;
  font-size: 28px;
  line-height: 1.2;
  font-weight: 700;
  color: #111827;
}
</style>
