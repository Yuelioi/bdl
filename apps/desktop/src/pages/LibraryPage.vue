<script setup lang="ts">
import { coverUrl } from "../utils/coverUrl";
import UiButton from '../ui/Button.vue'
import UiEmptyState from '../ui/EmptyState.vue'
import UiIconButton from '../ui/IconButton.vue'
import UiPagination from '../ui/Pagination.vue'
import UiTabs from '../ui/Tabs.vue'
import ExternalLinkButton from '../ui/ExternalLinkButton.vue'
import LibraryFolderDetail from './library/LibraryFolderDetail.vue'
import ParseDownloadPlanner from './parse/ParseDownloadPlanner.vue'
import { useLibraryPage } from "./useLibraryPage";
const { account, library, ui, query, failedCoverIds, detailFolder, detailLoading, activeCategory, activeCategoryUrl, categoryFilter, categoryTabs, page, visibleItems, load, openFolder, closeFolder, openDownloadSettings } = useLibraryPage();
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
        :loading-initial="detailLoading"
        @back="closeFolder"
        @download="openDownloadSettings"
      />

      <section v-else class="library-content" aria-labelledby="library-heading" :aria-busy="library.loading">
        <header class="library-header">
          <div>
            <div class="library-heading-line">
              <h2 id="library-heading">
                <ExternalLinkButton
                  v-if="activeCategoryUrl"
                  class="library-heading-link"
                  :href="activeCategoryUrl"
                  :label="`在 Bilibili 打开我的${activeCategory.label}`"
                  :show-icon="false"
                >
                  {{ activeCategory.label }}
                </ExternalLinkButton>
                <span v-else>{{ activeCategory.label }}</span>
              </h2>
            </div>
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
          description="账号凭据会加密持久化；macOS 使用本地加密凭据文件，内容库不会显示或导出 Cookie。"
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
          <article
            v-for="item in visibleItems"
            :key="item.media_id"
            class="library-card"
            role="button"
            tabindex="0"
            :aria-label="`进入 ${item.title}`"
            @click="openFolder(item)"
            @keydown.enter="openFolder(item)"
            @keydown.space.prevent="openFolder(item)"
          >
            <div class="library-cover">
              <img
referrerpolicy="no-referrer"
                v-if="item.cover_url && !failedCoverIds.includes(item.media_id)"
                :src="coverUrl(item.cover_url) ?? undefined"
                alt=""
                loading="lazy"
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
