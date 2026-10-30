<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiButton from './Button.vue'
import { openExternalUrl } from '../api/tauri'
import { captureDiagnosticEnvironment } from '../utils/diagnosticEnvironment'
import { redactDiagnosticMessage } from '../stores/transferView'
import { classifyFailure } from '../utils/failureDiagnostic'
import { writeClipboardText } from '../utils/clipboard'

const props = defineProps<{ detail: string; showAdvice?: boolean }>()
const detail = computed(() => redactDiagnosticMessage(props.detail))
const diagnosis = computed(() => props.showAdvice === false ? undefined : classifyFailure(detail.value))
const copied = ref(false)
const busy = ref(false)
const actionError = ref('')
watch(() => props.detail, () => { copied.value = false; actionError.value = '' })
const copy = async () => {
  busy.value = true
  copied.value = false
  actionError.value = ''
  const failure = detail.value
  try {
    const environment = await captureDiagnosticEnvironment()
    await writeClipboardText(redactDiagnosticMessage(`${failure}\n\n${environment}`))
    if (failure === detail.value) copied.value = true
  } catch {
    actionError.value = '复制失败，请展开错误详情后手动复制。'
  } finally {
    busy.value = false
  }
}
const feedback = async () => {
  actionError.value = ''
  try {
    await openExternalUrl('https://github.com/Yuelioi/bdl/issues/new/choose')
  } catch {
    actionError.value = '无法打开反馈页面，请访问 github.com/Yuelioi/bdl/issues。'
  }
}
</script>

<template>
  <div class="error-details">
    <p v-if="diagnosis">{{ diagnosis.summary }}：{{ diagnosis.detail }}</p>
    <details>
      <summary>错误详情</summary>
      <pre>{{ detail }}</pre>
    </details>
    <div class="error-actions">
      <UiButton size="compact" variant="secondary" :disabled="busy" @click="copy">{{ copied ? '已复制' : '复制诊断信息' }}</UiButton>
      <UiButton size="compact" variant="secondary" @click="feedback">反馈问题</UiButton>
    </div>
    <p>反馈时请附上诊断信息和操作步骤。</p>
    <p v-if="actionError" role="status">{{ actionError }}</p>
  </div>
</template>

<style scoped>
.error-details {
  min-width: 0;
  display: grid;
  gap: var(--space-8);
  overflow-wrap: anywhere;
}

.error-details p {
  margin: 0;
}

summary {
  cursor: pointer;
}

pre {
  max-height: 15rem;
  overflow: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  user-select: text;
  margin-block: var(--space-8);
}

.error-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-8);
}
</style>
