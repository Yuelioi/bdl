<script setup lang="ts">
import { computed, onMounted, ref, useTemplateRef, watch } from 'vue'

import type { AccountLibraryFolder, AccountLibraryFolderKind } from '../api/dto'
import { useAccountStore } from '../stores/account'
import { useLibraryStore } from '../stores/library'
import { useParseStore } from '../stores/parse'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiEmptyState from '../ui/EmptyState.vue'
import UiIconButton from '../ui/IconButton.vue'
import UiPagination from '../ui/Pagination.vue'
import UiTabs from '../ui/Tabs.vue'
import LibraryFolderDetail from './library/LibraryFolderDetail.vue'
import ParseDownloadPlanner from './parse/ParseDownloadPlanner.vue'

const account = useAccountStore()
const library = useLibraryStore()
const parse = useParseStore()
const ui = useUiStore()
const query = ref('')
const failedCoverIds = ref<string[]>([])
const detailFolder = ref<AccountLibraryFolder | null>(null)
const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner')

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
const categoryFilter = computed({
  get: () => library.activeKind,
  set: (value: string) => void selectCategory(value as AccountLibraryFolderKind),
})
const categoryTabs = computed(() =>
  categories.map((category) => ({
    label: category.label,
    value: category.kind,
    count: library.pages[category.kind]?.total,
  })),
)
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
  if (parse.activeSourceId) await parse.removeSource(parse.activeSourceId)
  detailFolder.value = null
  query.value = ''
  await library.selectKind(kind)
}

const openFolder = async (item: AccountLibraryFolder) => {
  await parse.createSource(item.source_url)
  if (parse.activeSource) {
    detailFolder.value = item
    parse.clearNotice()
  }
}

const closeFolder = async () => {
  if (parse.activeSourceId) await parse.removeSource(parse.activeSourceId)
  detailFolder.value = null
}

const openDownloadSettings = () => void downloadPlanner.value?.openDialog()

onMounted(() => {
  if (account.profile.logged_in) void load()
})

watch(
  () => [account.profile.logged_in, account.profile.mid] as const,
  ([loggedIn, mid], previous) => {
    if (!loggedIn) {
      library.clear()
      detailFolder.value = null
      return
    }
    if (!previous?.[0] || previous[1] !== mid) void load()
  },
)
</script>

<template>
  <section class="page-grid grid-cols-1" aria-label="账号内容库">
    <section class="panel library-panel">
      <div class="shrink-0">
        <UiTabs v-model="categoryFilter" :tabs="categoryTabs" />
      </div>

      <LibraryFolderDetail
        v-if="detailFolder"
        :folder="detailFolder"
        @back="closeFolder"
        @download="openDownloadSettings"
      />

      <section v-else class="library-content" aria-labelledby="library-heading" :aria-busy="library.loading">
        <header class="library-header">
          <div>
            <h2 id="library-heading">{{ activeCategory.label }}</h2>
            <p v-if="page">{{ activeCategory.description }} · 共 {{ page.total }} 个内容集合</p>
            <p v-else>选择内容集合，随后可继续筛选具体视频。</p>
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
                <button type="button" @click="openFolder(item)">
                  查看内容 <UIcon name="i-tabler-arrow-right" aria-hidden="true" />
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
          <template v-if="query" #action
            ><UiButton variant="secondary" @click="query = ''">清除搜索</UiButton></template
          >
        </UiEmptyState>

        <footer v-if="page && page.total > page.page_size" class="library-pagination">
          <UiPagination
            :page="page.page"
            :total="page.total"
            :items-per-page="page.page_size"
            :disabled="library.loading"
            :label="`${activeCategory.label}分页`"
            @update:page="load(library.activeKind, $event)"
          />
        </footer>
      </section>
    </section>
    <ParseDownloadPlanner ref="download-planner" />
  </section>
</template>

<style scoped src="./LibraryPage.css"></style>
