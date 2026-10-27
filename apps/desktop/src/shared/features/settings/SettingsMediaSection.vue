<script setup lang="ts">
import SettingsQualitySection from './SettingsQualitySection.vue';
import SettingsCard from './SettingsCard.vue';
import UiIconButton from '../../ui/IconButton.vue';
import type { SettingsForm } from './useSettingsForm';
const { form } = defineProps<{ form: SettingsForm }>();
</script>

<template>
  <SettingsCard title="画质与编码">
    <template #title-actions>
      <UPopover :content="{ align: 'start', sideOffset: 6, collisionPadding: 12 }">
        <UiIconButton icon="?" label="画质与编码说明" size="compact" variant="ghost" />
        <template #content>
          <section class="quality-help" aria-label="画质与编码说明">
            <h4>选择与编排</h4>
            <dl>
              <div>
                <dt>最佳可用</dt>
                <dd>按编排顺序选择首个可用档位。</dd>
              </div>
              <div>
                <dt>手动选档</dt>
                <dd>按所选画质或音质处理；缺少时遵循“指定档位不可用时”的设置。</dd>
              </div>
              <div>
                <dt>编码优先级</dt>
                <dd>先选画质或音质，同档位再按编码顺序选择。</dd>
              </div>
              <div>
                <dt>仅 SDR</dt>
                <dd>SDR 是普通动态范围，不是清晰度等级，也可以是 4K、8K；不选择 HDR 或杜比视界。</dd>
              </div>
              <div>
                <dt>调整顺序</dt>
                <dd>点“编排”，用上下箭头排序；下拉立即同步，点“保存”后生效。</dd>
              </div>
            </dl>
          </section>
        </template>
      </UPopover>
    </template>
    <SettingsQualitySection :form="form" />
  </SettingsCard>
</template>

<style scoped>
.quality-help {
  width: min(22rem, calc(100vw - 1.5rem));
  padding: var(--space-16);
  font-size: var(--font-12);
  line-height: 1.65;
}

.quality-help h4 {
  margin: 0 0 var(--space-12);
  font-size: var(--font-13);
}

.quality-help dl,
.quality-help dd {
  margin: 0;
}

.quality-help dl {
  display: grid;
  gap: var(--space-10);
}

.quality-help dt {
  font-weight: 600;
}

.quality-help dd {
  color: var(--color-muted);
}
</style>
