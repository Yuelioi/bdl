<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import type { AccountLibraryFolder, AccountLibraryFolderKind } from '../api/dto'
import { useAccountStore } from '../stores/account'
import { useLibraryStore } from '../stores/library'
import { useParseStore } from '../stores/parse'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiEmptyState from '../ui/EmptyState.vue'
import UiIconButton from '../ui/IconButton.vue'

const account = useAccountStore()
const library = useLibraryStore()
const parse = useParseStore()
const ui = useUiStore()
const query = ref('')
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
    [item.title, item.owner_name, item.description]
      .filter(Boolean)
      .some((value) => value!.toLocaleLowerCase().includes(keyword)),
  )
})
const load = async (kind = library.activeKind, targetPage = 1) => {
  await library.load(kind, targetPage)
}

const selectCategory = async (kind: AccountLibraryFolderKind) => {
  query.value = ''
  await library.selectKind(kind)
}

const openItem = async (item: AccountLibraryFolder) => {
  await parse.createSource(item.source_url)
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
      </div>

      <div v-if="library.loading && page" class="library-sync" role="status" aria-live="polite">
        <span aria-hidden="true"></span>
        正在同步{{ activeCategory.label }}…
      </div>

      <UiEmptyState
        v-if="!account.profile.logged_in"
        title="登录后连接你的内容库"
        description="账号凭据仍保存在系统安全存储中，内容库不会显示或导出 Cookie。"
        icon="i-tabler-lock"
      >
        <template #action><UiButton @click="ui.openLoginDialog()">登录账号</UiButton></template>
      </UiEmptyState>

      <UiEmptyState
        v-else-if="library.error"
        title="暂时无法读取内容库"
        :description="library.error"
        icon="i-tabler-cloud-off"
        tone="warning"
        role="alert"
      >
        <template #action><UiButton variant="secondary" @click="load()">重新加载</UiButton></template>
      </UiEmptyState>

      <div v-else-if="library.loading && !page" class="library-grid" aria-label="正在加载内容">
        <div v-for="index in 6" :key="index" class="library-card skeleton" aria-hidden="true">
          <span></span>
          <div><i></i><i></i><i></i></div>
        </div>
      </div>

      <div v-else-if="visibleItems.length" class="library-grid">
        <article v-for="item in visibleItems" :key="item.media_id" class="library-card">
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
              <button type="button" @click="openItem(item)">
                解析内容 <UIcon name="i-tabler-arrow-up-right" aria-hidden="true" />
              </button>
            </div>
          </div>
        </article>
      </div>

      <UiEmptyState
        v-else-if="page"
        :title="query ? '没有匹配的内容' : '这里还是空的'"
        :description="query ? '尝试缩短关键词，或清除搜索条件。' : '在 Bilibili 添加内容后，回到这里刷新即可。'"
        icon="i-tabler-folder-open"
      >
        <template v-if="query" #action><UiButton variant="secondary" @click="query = ''">清除搜索</UiButton></template>
      </UiEmptyState>

      <footer v-if="page && page.total > page.page_size" class="library-pagination" aria-label="内容库分页">
        <UiButton
          variant="ghost"
          :disabled="page.page <= 1 || library.loading"
          @click="load(library.activeKind, page.page - 1)"
        >
          上一页
        </UiButton>
        <span>第 {{ page.page }} 页</span>
        <UiButton
          variant="ghost"
          :disabled="!page.has_more || library.loading"
          @click="load(library.activeKind, page.page + 1)"
        >
          下一页
        </UiButton>
      </footer>
    </section>
  </section>
</template>

<style scoped src="./LibraryPage.css"></style>
