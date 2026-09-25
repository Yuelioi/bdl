<script setup lang="ts">
import { ref } from 'vue';
import { coverUrl } from '../../utils/coverUrl';
import UiButton from '../../ui/Button.vue';
import UiEmptyState from '../../ui/EmptyState.vue';
import UiIconButton from '../../ui/IconButton.vue';
import UiPagination from './MobilePagination.vue';
import UiTabs from './MobileTabs.vue';
import LibraryFolderDetail from './LibraryFolderDetail.vue';
import ParseDownloadPlanner from '../parse/ParseDownloadPlanner.vue';
import { useLibraryPage } from '../useLibraryPage';
const {
  account,
  library,
  ui,
  query,
  failedCoverIds,
  detailFolder,
  detailLoading,
  activeCategory,
  categoryFilter,
  categoryTabs,
  page,
  visibleItems,
  load,
  openFolder,
  closeFolder,
  openDownloadSettings,
} = useLibraryPage();
const searchOpen = ref(false);
</script>
<template>
  <section class="mobile-page" aria-label="账号内容库">
    <section class="library-panel">
      <div v-if="!detailFolder" class="library-topbar">
        <UiTabs v-model="categoryFilter" :tabs="categoryTabs" />
        <div v-if="account.profile.logged_in" class="library-actions">
          <button
            type="button"
            class="mobile-icon-button"
            aria-label="搜索内容库"
            :aria-pressed="searchOpen"
            @click="searchOpen = !searchOpen"
          >
            <UIcon name="i-tabler-search" />
          </button>
          <UiIconButton
            class="library-action"
            icon="refresh"
            label="刷新内容库"
            variant="ghost"
            :disabled="library.loading"
            @click="load()"
          />
        </div>
      </div>

      <LibraryFolderDetail
        v-if="detailFolder"
        :folder="detailFolder"
        :loading-initial="detailLoading"
        @back="closeFolder"
        @download="openDownloadSettings"
      />

      <section v-else class="library-content" aria-labelledby="library-heading" :aria-busy="library.loading">
        <h1 id="library-heading" class="sr-only">{{ activeCategory.label }}</h1>

        <div v-if="account.profile.logged_in && searchOpen" class="library-toolbar">
          <label class="library-search">
            <span class="sr-only">搜索当前分类</span>
            <UIcon name="i-tabler-search" aria-hidden="true" />
            <input v-model="query" type="search" placeholder="搜索标题或创建者" />
            <button
              type="button"
              aria-label="关闭搜索"
              @click="
                query = '';
                searchOpen = false;
              "
            >
              <UIcon name="i-tabler-x" />
            </button>
          </label>
        </div>

        <div v-if="library.loading && page" class="library-sync" role="status" aria-live="polite">
          <span aria-hidden="true"></span>
          正在同步{{ activeCategory.label }}…
        </div>

        <UiEmptyState
          v-if="!account.profile.logged_in"
          title="登录后连接你的内容库"
          description="浏览收藏夹和订阅合集，选择喜欢的视频下载。"
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
            <div><i></i><i></i></div>
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
              </div>
            </div>
            <div class="library-card-copy">
              <h3 :title="item.title">{{ item.title }}</h3>
              <div class="library-card-meta">
                <span class="library-owner">{{ item.owner_name || '我的收藏夹' }}</span>
                <span class="library-count">{{ item.media_count }} 个视频</span>
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
