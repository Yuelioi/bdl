import { computed, onDeactivated, onMounted, ref, useTemplateRef, watch } from 'vue';
import type { AccountLibraryFolder, AccountLibraryFolderKind } from '../api/dto';
import { useAccountStore } from '../stores/account';
import { useLibraryStore } from '../stores/library';
import { useParseStore } from '../stores/parse';
import { useUiStore } from '../stores/ui';
import { bilibiliFavoriteCategoryUrl } from '../utils/bilibiliLinks';

export function useLibraryPage() {
  const account = useAccountStore();
  const library = useLibraryStore();
  const parse = useParseStore();
  const ui = useUiStore();
  const query = ref('');
  const failedCoverIds = ref<string[]>([]);
  const detailFolder = ref<AccountLibraryFolder | null>(null);
  const detailLoading = ref(false);
  const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner');
  let detailRequestId = 0;
  const categories: Array<{
    kind: AccountLibraryFolderKind;
    label: string;
    description: string;
    icon: string;
  }> = [
    { kind: 'created_favorite', label: '收藏夹', description: '我创建的', icon: 'i-tabler-bookmarks' },
    { kind: 'collected_favorite', label: '订阅合集', description: '我收藏的', icon: 'i-tabler-folders' },
  ];
  const activeCategory = computed(() => categories.find((item) => item.kind === library.activeKind) ?? categories[0]);
  const activeCategoryUrl = computed(() => bilibiliFavoriteCategoryUrl(account.profile.mid, library.activeKind));
  const categoryFilter = computed({
    get: () => library.activeKind,
    set: (value: string) => void selectCategory(value as AccountLibraryFolderKind),
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
    await library.load(kind, targetPage);
  };
  const selectCategory = async (kind: AccountLibraryFolderKind) => {
    if (detailFolder.value || detailLoading.value) await closeFolder();
    query.value = '';
    await library.selectKind(kind);
  };
  const loadFolder = async (item: AccountLibraryFolder, requestId: number) => {
    const opened = await parse.createSource(item.source_url);
    if (requestId !== detailRequestId) {
      if (opened && parse.activeSource?.source.input === item.source_url && parse.activeSourceId) {
        await parse.clearWorkspace();
      }
      return;
    }
    detailLoading.value = false;
    if (opened) parse.clearNotice();
  };
  const openFolder = (item: AccountLibraryFolder) => {
    const requestId = ++detailRequestId;
    detailFolder.value = item;
    detailLoading.value = true;
    parse.clearNotice();
    void loadFolder(item, requestId);
  };
  const closeFolder = async () => {
    detailRequestId += 1;
    detailFolder.value = null;
    detailLoading.value = false;
    await parse.clearWorkspace();
  };
  const openDownloadSettings = () => void downloadPlanner.value?.openDialog();
  onMounted(() => {
    if (account.profile.logged_in) void load();
  });
  onDeactivated(() => {
    if (detailFolder.value || detailLoading.value) void closeFolder();
  });
  watch(
    () => [account.profile.logged_in, account.profile.mid] as const,
    ([loggedIn, mid], previous) => {
      if (!loggedIn) {
        if (detailFolder.value || detailLoading.value) void closeFolder();
        library.clear();
        return;
      }
      if (!previous?.[0] || previous[1] !== mid) void load();
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
