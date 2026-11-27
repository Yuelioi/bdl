<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiButton from './Button.vue'
import { openExternalUrl } from '../api/tauri'
import { captureDiagnosticEnvironment } from '../utils/diagnosticEnvironment'
import { redactDiagnosticMessage } from '../stores/transferView'
import { classifyFailure, failureGuidance, failureRule } from '../utils/failureDiagnostic'
import { writeClipboardText } from '../utils/clipboard'
import { useUiStore } from '../stores/ui'
import { isMobilePlatform } from '../platform/environment'

const props = withDefaults(defineProps<{ detail: string; showAdvice?: boolean }>(), { showAdvice: true })
const emit = defineEmits<{ openSettings: [] }>()
const ui = useUiStore()
const mobile = isMobilePlatform()
const detail = computed(() => redactDiagnosticMessage(props.detail))
const diagnosis = computed(() => {
  if (props.showAdvice === false) return undefined
  const code = detail.value.match(/^错误码: ([a-z_]+)$/m)?.[1]
  return (code && failureRule(code)) || classifyFailure(detail.value)
})
const guidance = computed(() => diagnosis.value && failureGuidance(diagnosis.value, 'parse', mobile))
const advice = computed(() => {
  if (!diagnosis.value) return ''
  const { summary, detail: advice } = diagnosis.value
  const message = `${summary}。${advice}`
  if (detail.value.split('\n').includes(message)) return ''
  if (detail.value.includes(`${summary}。解析已暂停`)) {
    return diagnosis.value.code === 'source_verification_required' ? '若 B 站网页要求验证，请先完成验证。' : ''
  }
  return message
})
const followGuidance = () => {
  if (!guidance.value) return
  const { section } = guidance.value
  if (section) {
    emit('openSettings')
    ui.openSettingsSection(section, section === 'settings-download')
  } else ui.openLoginDialog()
}
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
    <p v-if="advice || guidance?.tip">{{ advice }}{{ guidance?.tip }}</p>
    <UiButton v-if="guidance" class="recovery-action justify-self-start" :class="{ 'mobile-recovery': mobile }" :size="mobile ? 'default' : 'compact'" variant="secondary" @click="followGuidance">
      {{ guidance.label }}
    </UiButton>
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

.recovery-action.mobile-recovery {
  min-height: 3rem;
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
