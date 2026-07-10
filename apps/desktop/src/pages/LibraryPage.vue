<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import type { AccountLibraryFolder, AccountLibraryFolderKind } from '../api/dto'
import { useAccountStore } from '../stores/account'
import { useLibraryStore } from '../stores/library'
import { useParseStore } from '../stores/parse'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiIconButton from '../ui/IconButton.vue'

const account = useAccountStore()
const library = useLibraryStore()
const parse = useParseStore()
const ui = useUiStore()
const query = ref('')
const selectedIds = ref<string[]>([])
const failedCoverIds = ref<string[]>([])

const categories: Array<{
  kind: AccountLibraryFolderKind
  label: string
  description: string
  icon: string
}> = [
  { kind: 'created_favorite', label: '收藏夹', description: '我创建的', icon: 'i-tabler-bookmarks' },
  { kind: 'collected_favorite', label: '订阅合集', description: '我收藏的', icon: 'i-tabler-folders' },
]

const activeCategory = computed(() => categories.find((item) => item.kind === library.activeKind) ?? categories[0])
const page = computed(() => library.activePage)
const visibleItems = computed(() => {
  const keyword = query.value.trim().toLocaleLowerCase()
  if (!keyword) return page.value?.items ?? []
  return (page.value?.items ?? []).filter((item) =>
    [item.title, item.owner_name, item.description].filter(Boolean).some((value) => value!.toLocaleLowerCase().includes(keyword)),
  )
})
const selectedItems = computed(() => {
  const ids = new Set(selectedIds.value)
  return (page.value?.items ?? []).filter((item) => ids.has(item.media_id))
})

const load = async (kind = library.activeKind, targetPage = 1) => {
  selectedIds.value = []
  await library.load(kind, targetPage)
}

const selectCategory = async (kind: AccountLibraryFolderKind) => {
  query.value = ''
  selectedIds.value = []
  await library.selectKind(kind)
}

const toggleItem = (mediaId: string) => {
  selectedIds.value = selectedIds.value.includes(mediaId)
    ? selectedIds.value.filter((id) => id !== mediaId)
    : [...selectedIds.value, mediaId]
}

const openItems = async (items: AccountLibraryFolder[]) => {
  if (items.length === 0) return
  await parse.createSource(items.map((item) => item.source_url).join('\n'))
  ui.setTab('parse')
}

onMounted(() => {
  if (account.profile.logged_in) void load()
})

watch(
  () => [account.profile.logged_in, account.profile.mid] as const,
  ([loggedIn, mid], previous) => {
    if (!loggedIn) {
      library.clear()
      selectedIds.value = []
      return
    }
    if (!previous?.[0] || previous[1] !== mid) void load()
  },
)
</script>

<template>
  <section class="library-workspace" aria-label="账号内容库">
    <aside class="library-navigation" aria-label="内容库分类">
      <div class="library-intro">
        <span class="eyebrow">ACCOUNT LIBRARY</span>
        <h2>账号内容</h2>
        <p>从账号内容直接进入解析流程。</p>
      </div>

      <nav>
        <button
          v-for="category in categories"
          :key="category.kind"
          type="button"
          :class="{ active: library.activeKind === category.kind }"
          :aria-current="library.activeKind === category.kind ? 'page' : undefined"
          @click="selectCategory(category.kind)"
        >
          <UIcon :name="category.icon" aria-hidden="true" />
          <span>
            <strong>{{ category.label }}</strong>
            <small>{{ category.description }}</small>
          </span>
          <span v-if="library.pages[category.kind]" class="category-count">
            {{ library.pages[category.kind]?.total }}
          </span>
        </button>
      </nav>

      <div class="library-account">
        <span class="account-dot" :class="{ online: account.profile.logged_in }" aria-hidden="true"></span>
        <span>
          <strong>{{ account.profile.logged_in ? account.displayName : '尚未登录' }}</strong>
          <small>{{ account.profile.logged_in ? '账号内容已连接' : '登录后浏览个人内容' }}</small>
        </span>
      </div>
    </aside>

    <section class="library-content" aria-labelledby="library-heading" :aria-busy="library.loading">
      <header class="library-header">
        <div>
          <span class="eyebrow">{{ activeCategory.description }}</span>
          <h2 id="library-heading">{{ activeCategory.label }}</h2>
          <p v-if="page">共 {{ page.total }} 个内容集合</p>
          <p v-else>选择收藏夹，随后可继续筛选具体视频。</p>
        </div>
        <UiIconButton
          icon="refresh"
          label="刷新内容库"
          variant="ghost"
          :disabled="library.loading || !account.profile.logged_in"
          @click="load()"
        />
      </header>

      <div v-if="account.profile.logged_in" class="library-toolbar">
        <label class="library-search">
          <span class="sr-only">搜索当前分类</span>
          <UIcon name="i-tabler-search" aria-hidden="true" />
          <input v-model="query" type="search" placeholder="搜索标题、创建者或简介" />
        </label>
        <span class="toolbar-summary">已选 {{ selectedItems.length }}</span>
      </div>

      <div v-if="library.loading && page" class="library-sync" role="status" aria-live="polite">
        <span aria-hidden="true"></span>
        正在同步{{ activeCategory.label }}…
      </div>

      <div v-if="!account.profile.logged_in" class="library-empty library-login-state">
        <span class="empty-symbol"><UIcon name="i-tabler-lock" aria-hidden="true" /></span>
        <div>
          <h3>登录后连接你的内容库</h3>
          <p>账号凭据仍保存在系统安全存储中，内容库不会显示或导出 Cookie。</p>
        </div>
        <UiButton @click="ui.openLoginDialog()">登录账号</UiButton>
      </div>

      <div v-else-if="library.error" class="library-empty" role="alert">
        <span class="empty-symbol warning"><UIcon name="i-tabler-cloud-off" aria-hidden="true" /></span>
        <div>
          <h3>暂时无法读取内容库</h3>
          <p>{{ library.error }}</p>
        </div>
        <UiButton variant="secondary" @click="load()">重新加载</UiButton>
      </div>

      <div v-else-if="library.loading && !page" class="library-grid" aria-label="正在加载内容">
        <div v-for="index in 6" :key="index" class="library-card skeleton" aria-hidden="true">
          <span></span><div><i></i><i></i><i></i></div>
        </div>
      </div>

      <div v-else-if="visibleItems.length" class="library-grid">
        <article
          v-for="item in visibleItems"
          :key="item.media_id"
          class="library-card"
          :class="{ selected: selectedIds.includes(item.media_id) }"
        >
          <button
            class="library-check"
            type="button"
            :aria-pressed="selectedIds.includes(item.media_id)"
            :aria-label="`${selectedIds.includes(item.media_id) ? '取消选择' : '选择'} ${item.title}`"
            @click="toggleItem(item.media_id)"
          >
            <UIcon :name="selectedIds.includes(item.media_id) ? 'i-tabler-check' : 'i-tabler-plus'" aria-hidden="true" />
          </button>
          <div class="library-cover">
            <img
              v-if="item.cover_url && !failedCoverIds.includes(item.media_id)"
              :src="item.cover_url"
              alt=""
              loading="lazy"
              referrerpolicy="no-referrer"
              @error="failedCoverIds = [...failedCoverIds, item.media_id]"
            />
            <div v-else class="cover-fallback" aria-hidden="true">
              <UIcon name="i-tabler-bookmark" />
              <span>{{ item.media_count }}</span>
            </div>
          </div>
          <div class="library-card-copy">
            <div>
              <h3 :title="item.title">{{ item.title }}</h3>
              <p>{{ item.owner_name || '我的收藏夹' }}</p>
            </div>
            <p v-if="item.description" class="library-description">{{ item.description }}</p>
            <div class="library-card-meta">
              <span>{{ item.media_count }} 个视频</span>
              <button type="button" @click="openItems([item])">
                打开内容 <UIcon name="i-tabler-arrow-up-right" aria-hidden="true" />
              </button>
            </div>
          </div>
        </article>
      </div>

      <div v-else-if="page" class="library-empty">
        <span class="empty-symbol"><UIcon name="i-tabler-folder-open" aria-hidden="true" /></span>
        <div>
          <h3>{{ query ? '没有匹配的内容' : '这里还是空的' }}</h3>
          <p>{{ query ? '尝试缩短关键词，或清除搜索条件。' : '在 Bilibili 添加内容后，回到这里刷新即可。' }}</p>
        </div>
        <UiButton v-if="query" variant="secondary" @click="query = ''">清除搜索</UiButton>
      </div>

      <footer v-if="page && page.total > page.page_size" class="library-pagination" aria-label="内容库分页">
        <UiButton variant="ghost" :disabled="page.page <= 1 || library.loading" @click="load(library.activeKind, page.page - 1)">
          上一页
        </UiButton>
        <span>第 {{ page.page }} 页</span>
        <UiButton variant="ghost" :disabled="!page.has_more || library.loading" @click="load(library.activeKind, page.page + 1)">
          下一页
        </UiButton>
      </footer>

      <Transition name="selection-bar">
        <div v-if="selectedItems.length" class="library-selection-bar" role="region" aria-label="已选内容操作">
          <span><strong>{{ selectedItems.length }}</strong> 个集合已选</span>
          <div>
            <UiButton variant="ghost" @click="selectedIds = []">取消选择</UiButton>
            <UiButton @click="openItems(selectedItems)">解析所选</UiButton>
          </div>
        </div>
      </Transition>
    </section>
  </section>
</template>

<style scoped>
.library-workspace {
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-columns: 196px minmax(0, 1fr);
  overflow: hidden;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-12);
  background: var(--color-surface);
  box-shadow: var(--shadow-panel);
}

.library-navigation {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  padding: var(--space-lg) var(--space-sm) var(--space-md);
  border-right: 1px solid var(--color-border);
  background: var(--color-inset);
}

.library-intro { padding: 0 var(--space-xs); }
.eyebrow { color: var(--color-accent); font-family: var(--font-display); font-size: var(--font-11); font-weight: 750; letter-spacing: .08em; }
.library-intro h2, .library-header h2 { margin: var(--space-2xs) 0 0; color: var(--color-text-strong); font-family: var(--font-display); font-size: var(--font-22); line-height: 1.2; }
.library-intro p, .library-header p { margin: var(--space-xs) 0 0; color: var(--color-muted); font-size: var(--font-12); line-height: 1.55; }

.library-navigation nav { display: grid; gap: var(--space-2xs); }
.library-navigation nav button {
  min-height: 54px;
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-xs) var(--space-sm);
  border: 1px solid transparent;
  border-radius: var(--radius-8);
  color: var(--color-muted);
  background: transparent;
  text-align: left;
}
.library-navigation nav button:hover { color: var(--color-text); background: var(--color-hover-surface); }
.library-navigation nav button.active { color: var(--color-text-strong); border-color: var(--color-border); background: var(--color-surface-raised); }
.library-navigation nav svg { width: 18px; height: 18px; }
.library-navigation nav strong, .library-navigation nav small { display: block; }
.library-navigation nav strong { font-size: var(--font-13); }
.library-navigation nav small { margin-top: 2px; color: var(--color-dimmed); font-size: var(--font-11); font-weight: 500; }
.category-count { color: var(--color-dimmed); font-family: var(--font-display); font-size: var(--font-11); font-variant-numeric: tabular-nums; }

.library-account { margin-top: auto; display: flex; align-items: center; gap: var(--space-xs); padding: var(--space-sm) var(--space-xs) 0; border-top: 1px solid var(--color-border); }
.library-account strong, .library-account small { display: block; }
.library-account strong { color: var(--color-text); font-size: var(--font-12); }
.library-account small { margin-top: 2px; color: var(--color-dimmed); font-size: var(--font-11); }
.account-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--color-dimmed); }
.account-dot.online { background: var(--color-success); box-shadow: 0 0 0 4px color-mix(in oklab, var(--color-success) 14%, transparent); }

.library-content { position: relative; min-width: 0; min-height: 0; display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-lg); overflow: auto; }
.library-header { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--space-md); }
.library-header h2 { font-size: var(--font-18); }
.library-header p { margin-top: var(--space-2xs); }

.library-toolbar { display: flex; align-items: center; gap: var(--space-sm); }
.library-search { flex: 1; height: var(--height-input); display: grid; grid-template-columns: 18px minmax(0, 1fr); align-items: center; gap: var(--space-xs); padding: 0 var(--space-sm); border: 1px solid var(--color-border); border-radius: var(--radius-6); background: var(--color-inset); }
.library-search:focus-within { border-color: var(--color-accent); outline: 2px solid var(--color-focus-outline); }
.library-search svg { color: var(--color-dimmed); }
.library-search input { min-width: 0; border: 0; outline: 0; background: transparent; color: var(--color-text); font-size: var(--font-13); }
.toolbar-summary { color: var(--color-muted); font-size: var(--font-12); white-space: nowrap; }
.library-sync { display: flex; align-items: center; gap: var(--space-xs); color: var(--color-muted); font-size: var(--font-12); }
.library-sync span { width: 72px; height: 3px; overflow: hidden; border-radius: 999px; background: var(--color-progress-track); }
.library-sync span::after { content: ''; width: 45%; height: 100%; display: block; border-radius: inherit; background: var(--color-accent); animation: library-sync 1s var(--ease-out) infinite alternate; }

.library-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(310px, 100%), 1fr)); align-content: start; gap: var(--space-sm); }
.library-card { position: relative; min-width: 0; display: grid; grid-template-columns: 112px minmax(0, 1fr); gap: var(--space-sm); padding: var(--space-sm); border: 1px solid var(--color-border); border-radius: var(--radius-8); background: var(--color-surface-raised); transition: border-color var(--duration-fast) var(--ease-out), transform var(--duration-fast) var(--ease-out), background var(--duration-fast) var(--ease-out); }
.library-card:hover { border-color: var(--color-border-strong); transform: translateY(-1px); }
.library-card.selected { border-color: var(--color-accent); background: var(--color-selected-surface); }
.library-cover { aspect-ratio: 4 / 3; overflow: hidden; border-radius: var(--radius-6); background: var(--color-panel); }
.library-cover img { width: 100%; height: 100%; display: block; object-fit: cover; }
.cover-fallback { width: 100%; height: 100%; display: grid; place-content: center; justify-items: center; gap: var(--space-xs); color: var(--color-accent); background-image: linear-gradient(var(--color-grid-line) 1px, transparent 1px), linear-gradient(90deg, var(--color-grid-line) 1px, transparent 1px); background-size: 16px 16px; }
.cover-fallback svg { width: 24px; height: 24px; }
.cover-fallback span { font-family: var(--font-display); font-size: var(--font-12); font-weight: 700; }
.library-check { position: absolute; z-index: 1; top: var(--space-xs); left: var(--space-xs); width: 28px; height: 28px; display: grid; place-items: center; border: 1px solid var(--color-border); border-radius: 50%; color: var(--color-text); background: var(--color-surface-raised); box-shadow: var(--shadow-panel); }
.library-check[aria-pressed="true"] { border-color: var(--color-accent); color: var(--color-on-accent); background: var(--color-accent); }
.library-check svg { width: 14px; height: 14px; }
.library-card-copy { min-width: 0; display: flex; flex-direction: column; gap: var(--space-xs); }
.library-card-copy h3 { margin: 0; overflow: hidden; color: var(--color-text-strong); font-size: var(--font-14); line-height: 1.4; text-overflow: ellipsis; white-space: nowrap; }
.library-card-copy p { margin: 2px 0 0; color: var(--color-muted); font-size: var(--font-12); }
.library-description { display: -webkit-box; overflow: hidden; line-height: 1.5; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.library-card-meta { margin-top: auto; display: flex; align-items: center; justify-content: space-between; gap: var(--space-xs); color: var(--color-dimmed); font-size: var(--font-11); font-variant-numeric: tabular-nums; }
.library-card-meta button { display: inline-flex; align-items: center; gap: var(--space-2xs); padding: var(--space-2xs) 0; border: 0; color: var(--color-accent); background: transparent; font-size: var(--font-12); font-weight: 650; }
.library-card-meta button:hover { color: var(--color-accent-strong); }
.library-card-meta svg { width: 13px; height: 13px; }

.library-empty { min-height: 240px; flex: 1; display: flex; align-items: center; justify-content: center; gap: var(--space-md); padding: var(--space-xl); border: 1px dashed var(--color-border-strong); border-radius: var(--radius-8); background: var(--color-inset); }
.library-empty h3 { margin: 0; color: var(--color-text-strong); font-size: var(--font-16); }
.library-empty p { max-width: 52ch; margin: var(--space-2xs) 0 0; color: var(--color-muted); font-size: var(--font-13); line-height: 1.6; }
.empty-symbol { width: 44px; height: 44px; flex: 0 0 auto; display: grid; place-items: center; border-radius: 50%; color: var(--color-accent); background: var(--color-accent-soft); }
.empty-symbol.warning { color: var(--color-warning); background: var(--color-warning-soft); }
.empty-symbol svg { width: 20px; height: 20px; }

.library-pagination { display: flex; align-items: center; justify-content: center; gap: var(--space-sm); color: var(--color-muted); font-size: var(--font-12); }
.library-selection-bar { position: sticky; z-index: 2; bottom: 0; display: flex; align-items: center; justify-content: space-between; gap: var(--space-md); padding: var(--space-sm) var(--space-md); border: 1px solid var(--color-border-strong); border-radius: var(--radius-8); background: var(--color-surface-raised); box-shadow: var(--shadow-overlay); }
.library-selection-bar > div { display: flex; gap: var(--space-xs); }
.library-selection-bar span { color: var(--color-muted); font-size: var(--font-13); }
.library-selection-bar strong { color: var(--color-text-strong); font-family: var(--font-display); }
.selection-bar-enter-active, .selection-bar-leave-active { transition: opacity var(--duration-fast) var(--ease-out), transform var(--duration-fast) var(--ease-out); }
.selection-bar-enter-from, .selection-bar-leave-to { opacity: 0; transform: translateY(8px); }

.skeleton { pointer-events: none; }
.skeleton > span, .skeleton i { display: block; border-radius: var(--radius-4); background: var(--color-panel); animation: library-pulse 1.4s ease-in-out infinite alternate; }
.skeleton > span { aspect-ratio: 4 / 3; }
.skeleton > div { display: grid; align-content: center; gap: var(--space-xs); }
.skeleton i:nth-child(1) { width: 82%; height: 14px; }
.skeleton i:nth-child(2) { width: 48%; height: 10px; }
.skeleton i:nth-child(3) { width: 64%; height: 10px; }
@keyframes library-pulse { to { opacity: .48; } }
@keyframes library-sync { from { transform: translateX(-20%); } to { transform: translateX(120%); } }

.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }

@media (max-width: 860px) {
  .library-workspace { grid-template-columns: 1fr; overflow: auto; }
  .library-navigation { position: sticky; z-index: 2; top: 0; flex-direction: row; align-items: center; padding: var(--space-sm); border-right: 0; border-bottom: 1px solid var(--color-border); }
  .library-intro, .library-account { display: none; }
  .library-navigation nav { width: 100%; grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .library-navigation nav button { min-height: 44px; }
  .library-content { overflow: visible; }
}

@media (max-width: 560px) {
  .library-content { padding: var(--space-md); }
  .library-card { grid-template-columns: 88px minmax(0, 1fr); }
  .toolbar-summary { display: none; }
  .library-empty { flex-direction: column; text-align: center; }
  .library-selection-bar { align-items: stretch; flex-direction: column; }
  .library-selection-bar > div { justify-content: flex-end; }
}

@media (prefers-reduced-motion: reduce) {
  .library-card, .selection-bar-enter-active, .selection-bar-leave-active { transition: none; }
  .skeleton > span, .skeleton i { animation: none; }
  .library-sync span::after { animation: none; }
}
</style>
