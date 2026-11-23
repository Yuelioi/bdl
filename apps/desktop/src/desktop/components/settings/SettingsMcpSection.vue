<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { onMounted, ref } from 'vue';
import SettingsCard from '../../../shared/features/settings/SettingsCard.vue';
import UiButton from '../../../shared/ui/Button.vue';
import UiInlineNotice from '../../../shared/ui/InlineNotice.vue';
import { writeClipboardText } from '../../../shared/utils/clipboard';

type Connection = { enabled: boolean; url: string; error?: string | null };

const connection = ref<Connection>();
const busy = ref(false);
const error = ref('');
const copied = ref(false);

async function refresh() {
  busy.value = true;
  error.value = '';
  copied.value = false;
  try {
    connection.value = await invoke<Connection>('desktop_mcp_status');
  } catch {
    connection.value = undefined;
    error.value = '无法读取 AI 接入状态。';
  } finally {
    busy.value = false;
  }
}

onMounted(refresh);

async function setEnabled(enabled: boolean) {
  busy.value = true;
  error.value = '';
  copied.value = false;
  try {
    connection.value = await invoke<Connection>('desktop_mcp_set_enabled', { enabled });
  } catch (cause) {
    error.value = typeof cause === 'string' ? cause : '无法更改 AI 接入状态。';
  } finally {
    busy.value = false;
  }
}

async function copyConfig() {
  const current = connection.value;
  if (!current?.enabled) return;
  busy.value = true;
  try {
    const token = await invoke<string>('desktop_mcp_token');
    const server = {
      type: 'http',
      url: current.url,
      headers: { Authorization: `Bearer ${token}` },
    };
    await writeClipboardText(JSON.stringify({ mcpServers: { bdl: server } }, null, 2));
    error.value = '';
    copied.value = true;
  } catch {
    copied.value = false;
    error.value = '无法复制配置，请重试。';
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="settings-block">
    <SettingsCard>
      <p class="mcp-description">使用时请保持 BDL 运行。可连接支持 Streamable HTTP 的本机 MCP 客户端。</p>
      <template v-if="connection?.enabled">
        <div class="mcp-address">
          <span>连接地址</span>
          <code>{{ connection.url }}</code>
        </div>
        <p class="mcp-description">将配置粘贴到客户端的 MCP 设置。配置含接入令牌，请妥善保管。</p>
      </template>
      <UiInlineNotice v-if="error || connection?.error" tone="danger">{{ error || connection?.error }}</UiInlineNotice>
      <div class="mcp-actions">
        <template v-if="connection?.enabled">
          <UiButton variant="secondary" :disabled="busy" @click="setEnabled(false)">关闭接入</UiButton>
          <UiButton :disabled="busy" @click="copyConfig">
            <UIcon :name="copied ? 'i-tabler-check' : 'i-tabler-copy'" class="size-4" aria-hidden="true" />
            {{ copied ? '已复制' : '复制配置' }}
          </UiButton>
        </template>
        <template v-else>
          <UiButton v-if="connection?.error" variant="ghost" :disabled="busy" @click="setEnabled(false)"
            >重置接入</UiButton
          >
          <UiButton v-if="!connection && error" variant="secondary" :disabled="busy" @click="refresh">重试</UiButton>
          <UiButton :disabled="busy || !connection" @click="setEnabled(true)">开启接入</UiButton>
        </template>
      </div>
    </SettingsCard>
    <SettingsCard title="支持的操作">
      <ul class="mcp-tools">
        <li>解析 B 站链接、翻看分集，浏览已登录账号的内容库。</li>
        <li>查看下载预设，按所选预设把内容加入「传输」。</li>
        <li>浏览下载队列、查询状态，暂停、继续或重试任务。</li>
        <li>查看和修改下载总限速、并发数、解析速度及默认预设。</li>
      </ul>
    </SettingsCard>
  </div>
</template>

<style scoped>
.mcp-description {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-13);
  line-height: 1.6;
}

.mcp-address {
  display: grid;
  gap: var(--space-4);
  padding: var(--space-8) var(--space-12);
  border-radius: var(--radius-6);
  background-color: var(--color-inset);
  font-size: var(--font-13);
  overflow-wrap: anywhere;
  user-select: text;
}

.mcp-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-8);
}

.mcp-tools {
  display: grid;
  gap: var(--space-6);
  margin: 0;
  padding-inline-start: 1.25rem;
  list-style: disc;
  color: var(--color-muted);
  font-size: var(--font-13);
}
</style>
