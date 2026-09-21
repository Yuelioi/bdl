<script setup lang="ts">
defineEmits<{ openLink: [url: string] }>()

const links = [
  {
    label: 'GitHub 仓库',
    value: 'github.com/Yuelioi/bdl',
    url: 'https://github.com/Yuelioi/bdl',
    icon: 'i-tabler-brand-github',
  },
  {
    label: '作者 Bilibili',
    value: 'space.bilibili.com/4279370',
    url: 'https://space.bilibili.com/4279370',
    icon: 'i-tabler-brand-bilibili',
  },
]
</script>

<template>
  <div class="usage-notice">
    <p class="usage-intro">
      BDL 是用于解析、选择和下载 Bilibili 内容的本地工具。继续使用前，请确认你理解以下使用边界。
    </p>

    <section class="usage-section usage-section-free">
      <UIcon name="i-tabler-gift" aria-hidden="true" />
      <div>
        <strong>软件免费</strong>
        <p>BDL 软件本体免费，并以 MIT License 发布；官方仓库不设置激活码、会员版或付费解锁。</p>
      </div>
    </section>

    <section class="usage-section usage-section-abuse">
      <UIcon name="i-tabler-shield-exclamation" aria-hidden="true" />
      <div>
        <strong>禁止滥用</strong>
        <p>严禁用于批量搬运、侵权传播、商业盗用、绕过权限或访问控制、恶意刷取接口，以及任何违法用途。</p>
      </div>
    </section>

    <section class="usage-copy">
      <strong>使用责任与免责声明</strong>
      <p>
        BDL 不提供或托管视频内容。你应自行确认下载、保存、转换和使用内容具有合法授权，并遵守所在地法律法规、Bilibili 平台规则及内容权利人的要求。因具体使用方式产生的版权、账号、数据或其他风险由使用者自行承担。
      </p>
    </section>

    <section class="usage-links" aria-label="作者与项目链接">
      <button v-for="link in links" :key="link.url" type="button" @click="$emit('openLink', link.url)">
        <UIcon :name="link.icon" aria-hidden="true" />
        <span>
          <strong>{{ link.label }}</strong>
          <small>{{ link.value }}</small>
        </span>
        <UIcon name="i-tabler-external-link" class="usage-link-arrow" aria-hidden="true" />
      </button>
    </section>
  </div>
</template>

<style scoped>
.usage-notice {
  display: grid;
  gap: var(--space-12);
}

.usage-intro,
.usage-section p,
.usage-copy p {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-13);
  line-height: 1.65;
}

.usage-section {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: var(--space-10);
  padding: var(--space-12);
  border-radius: var(--radius-8);
}

.usage-section > svg {
  width: 20px;
  height: 20px;
  margin-top: 1px;
}

.usage-section > div,
.usage-copy {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.usage-section strong,
.usage-copy strong {
  color: var(--color-text-strong);
  font-size: var(--font-13);
}

.usage-section-free {
  background: var(--color-info-soft);
}

.usage-section-free > svg {
  color: var(--color-info);
}

.usage-section-abuse {
  background: var(--color-danger-soft);
}

.usage-section-abuse > svg,
.usage-section-abuse strong {
  color: var(--color-danger);
}

.usage-copy {
  padding: 0 var(--space-4);
}

.usage-links {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-8);
}

.usage-links button {
  min-width: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
  padding: var(--space-10);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
}

.usage-links button:hover,
.usage-links button:focus-visible {
  border-color: var(--color-border-strong);
  background: var(--color-hover-surface);
  outline: 0;
}

.usage-links button:focus-visible {
  box-shadow: inset 0 0 0 1px var(--color-focus-outline);
}

.usage-links button > svg {
  width: 18px;
  height: 18px;
  color: var(--color-accent-strong);
}

.usage-links button > span {
  min-width: 0;
  display: grid;
  gap: 1px;
}

.usage-links button strong,
.usage-links button small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-links button strong {
  font-size: var(--font-12);
}

.usage-links button small {
  color: var(--color-muted);
  font-size: var(--font-11);
}

.usage-links .usage-link-arrow {
  width: 14px;
  height: 14px;
  color: var(--color-muted);
}

@media (width <= 520px) {
  .usage-links {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
