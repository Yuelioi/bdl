import { computed, onActivated, onDeactivated, onMounted, ref, useTemplateRef, watch } from 'vue';
import type { LibraryContentSource, LibraryCategory } from '../../api/dto';
import type { ParseCancelReason } from '../../api/tauri';
import { useAccountStore } from '../../stores/account';
import { useLibraryStore } from '../../stores/library';
import { useParseStore } from '../../stores/parse';
import { useUiStore } from '../../stores/ui';
import { bilibiliFavoriteCategoryUrl } from '../../utils/bilibiliLinks';

export function useLibraryPage() {
  const account = useAccountStore();
  const library = useLibraryStore();
  const parse = useParseStore();
  const ui = useUiStore();
  const query = ref('');
  const failedCoverIds = ref<string[]>([]);
  const detailFolder = ref<LibraryContentSource | null>(null);
  const detailLoading = ref(false);
  const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner');
  let detailRequestId = 0;
  let pageActive = false;
  const categories: Array<{
    kind: LibraryCategory;
    label: string;
    description: string;
    icon: string;
  }> = [
    { kind: 'created_favorite', label: '我的收藏', description: '我创建的', icon: 'i-tabler-bookmarks' },
    { kind: 'collected_favorite', label: '订阅合集', description: '我收藏的', icon: 'i-tabler-folders' },
    { kind: 'watch_later', label: '稍后再看', description: '稍后观看的视频', icon: 'i-tabler-clock' },
  ];
  const activeCategory = computed(() => categories.find((item) => item.kind === library.activeKind) ?? categories[0]);
  const activeCategoryUrl = computed(() => library.activeKind === 'watch_later' ? 'https://www.bilibili.com/list/watchlater' : bilibiliFavoriteCategoryUrl(account.profile.mid, library.activeKind));
  const categoryFilter = computed({
    get: () => library.activeKind,
    set: (value: string) => void selectCategory(value as LibraryCategory),
  });
  const categoryTabs = computed(() =>
    categories.map((category) => ({
      label: category.label,
      value: category.kind,
      count: library.pages[category.kind]?.total,
    })),
  );
  const page = computed(() => library.activePage);
  const visibleItems = computed(() => {
    const keyword = query.value.trim().toLocaleLowerCase();
    if (!keyword) return page.value?.items ?? [];
    return (page.value?.items ?? []).filter((item) =>
      [item.title, item.owner_name, item.description]
        .filter(Boolean)
        .some((value) => value!.toLocaleLowerCase().includes(keyword)),
    );
  });
  const load = async (kind = library.activeKind, targetPage = 1) => {
    if (!pageActive || !account.profile.logged_in) return;
    await library.load(kind, targetPage);
  };
  const selectCategory = async (kind: LibraryCategory) => {
    if (detailFolder.value || detailLoading.value) await closeFolder();
    query.value = '';
    await library.selectKind(kind);
  };
  const loadFolder = async (item: LibraryContentSource, requestId: number) => {
    const opened = await parse.createSource(item.source_url);
    if (requestId !== detailRequestId) return;
    if (opened && parse.activeSource) {
      const resolved = parse.activeSource.source;
      detailFolder.value = { ...item, title: resolved.title, parsed_source_id: resolved.id,
        source_url: /^https?:\/\//.test(resolved.input) ? resolved.input : item.source_url };
    }
    detailLoading.value = false;
    if (opened) parse.clearNotice();
  };
  const openFolder = (item: LibraryContentSource) => {
    const requestId = ++detailRequestId;
    detailFolder.value = item;
    detailLoading.value = true;
    parse.clearNotice();
    void loadFolder(item, requestId);
  };
  const clearFolder = async (reason: ParseCancelReason) => {
    detailRequestId += 1;
    detailFolder.value = null;
    detailLoading.value = false;
    await parse.clearWorkspace(reason);
  };
  const closeFolder = () => clearFolder('library_folder_closed');
  const openDownloadSettings = () => void downloadPlanner.value?.openDialog();
  onMounted(() => {
    if (pageActive) return;
    pageActive = true;
    if (account.profile.logged_in) void load();
  });
  onActivated(() => {
    if (pageActive) return;
    pageActive = true;
    if (account.profile.logged_in) void load();
  });
  onDeactivated(() => {
    pageActive = false;
    if (detailFolder.value || detailLoading.value) void clearFolder('library_deactivated');
  });
  watch(
    () => [account.profile.logged_in, account.profile.mid] as const,
    ([loggedIn, mid], previous) => {
      if (!loggedIn) {
        if (detailFolder.value || detailLoading.value) void clearFolder('account_changed');
        library.clear();
        return;
      }
      if (!previous?.[0] || previous[1] !== mid) {
        library.clear();
        if (detailFolder.value || detailLoading.value) void clearFolder('account_changed').then(() => load());
        else void load();
      }
    },
  );
  return {
    account,
    library,
    ui,
    query,
    failedCoverIds,
    detailFolder,
    detailLoading,
    activeCategory,
    activeCategoryUrl,
    categoryFilter,
    categoryTabs,
    page,
    visibleItems,
    load,
    openFolder,
    closeFolder,
    openDownloadSettings,
  };
}
