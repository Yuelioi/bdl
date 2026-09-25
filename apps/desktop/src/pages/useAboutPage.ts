import { ref } from 'vue';
import { getVersion } from '@tauri-apps/api/app';
import { onMounted } from 'vue';
import { openExternalUrl } from '../api/tauri';
import { useUiStore } from '../stores/ui';

export function useAboutPage() {
  const ui = useUiStore();
  const version = ref('0.1.0');
  const links = [
    {
      label: 'GitHub 仓库',
      value: 'github.com/Yuelioi/bdl',
      url: 'https://github.com/Yuelioi/bdl',
      icon: 'i-tabler-brand-github',
    },
    {
      label: 'Bilibili 主页',
      value: 'space.bilibili.com/4279370',
      url: 'https://space.bilibili.com/4279370',
      icon: 'i-tabler-brand-bilibili',
    },
    {
      label: '个人网站',
      value: 'www.yuelili.com',
      url: 'https://www.yuelili.com',
      icon: 'i-tabler-world-www',
    },
  ];
  const openLink = async (url: string) => {
    try {
      await openExternalUrl(url);
    } catch (error) {
      ui.pushToast(error instanceof Error ? error.message : String(error), 'danger');
    }
  };
  onMounted(async () => {
    try {
      version.value = await getVersion();
    } catch {
      // The package version remains available in browser-only previews.
    }
  });
  return { version, links, openLink };
}
