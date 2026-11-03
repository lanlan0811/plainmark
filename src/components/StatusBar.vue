<script setup lang="ts">
/**
 * 状态栏。
 *
 * 信息与记事本一致（行/列、缩放、换行符、编码），但呈现方式不同：
 * 记事本是「灰底 + 竖线分隔 + 全直角」，这里改为「浅底 + 圆角胶囊字段」，
 * 可点击的字段有明确的悬停反馈，让「可切换」这件事一眼可见。
 */
import { ENCODING_LABELS, EOL_CHOICES, EOL_LABELS, type Encoding, type Eol } from "../types";

defineProps<{
  line: number;
  column: number;
  zoom: number;
  eol: Eol;
  encoding: Encoding;
  /** 无路径的新文档不允许切换编码 */
  hasPath: boolean;
}>();

const emit = defineEmits<{
  (e: "set-eol", eol: Eol): void;
  (e: "set-encoding", encoding: Encoding): void;
  (e: "set-zoom", zoom: number): void;
}>();

function onEolChange(event: Event) {
  emit("set-eol", (event.target as HTMLSelectElement).value as Eol);
}

function onEncodingChange(event: Event) {
  emit("set-encoding", (event.target as HTMLSelectElement).value as Encoding);
}
</script>

<template>
  <div class="statusbar">
    <div class="field field-static" title="光标位置">
      <span class="dot" aria-hidden="true"></span>
      第 {{ line }} 行, 第 {{ column }} 列
    </div>

    <div class="spacer"></div>

    <button
      class="field field-button"
      type="button"
      :title="`当前缩放 ${zoom}%，点击恢复 100%`"
      @click="emit('set-zoom', 100)"
    >
      {{ zoom }}%
    </button>

    <label class="field field-select" title="换行符">
      <select :value="eol" @change="onEolChange">
        <option v-for="choice in EOL_CHOICES" :key="choice" :value="choice">
          {{ EOL_LABELS[choice] }}
        </option>
      </select>
    </label>

    <!--
      新文档尚未落盘，编码由「另存为」决定，因此禁用。
      这里用 class 而非 CSS 的 :has() —— :has() 正好从 Chromium 105 才默认启用，
      而构建目标就是 105，踩在边界上不如显式控制稳妥。
    -->
    <label
      class="field field-select"
      :class="{ 'is-disabled': !hasPath }"
      title="编码"
    >
      <select :value="encoding" :disabled="!hasPath" @change="onEncodingChange">
        <option v-for="(label, key) in ENCODING_LABELS" :key="key" :value="key">
          {{ label }}
        </option>
      </select>
    </label>
  </div>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: var(--pm-statusbar-height);
  padding: 0 8px;
  background: var(--pm-chrome);
  border-top: 1px solid var(--pm-border);
  color: var(--pm-fg-muted);
  font-size: 11px;
  flex: none;
}

.spacer {
  flex: 1;
}

.field {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 20px;
  padding: 0 8px;
  border: 1px solid transparent;
  border-radius: var(--pm-radius-sm);
  white-space: nowrap;
  transition:
    background var(--pm-transition),
    border-color var(--pm-transition),
    color var(--pm-transition);
}

.field-static {
  /* 纯展示字段不加交互反馈，避免误导为可点击 */
  padding-left: 4px;
}

/* 状态点：与品牌色呼应的小圆点，替代记事本那条生硬的分隔线 */
.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--pm-brand);
  flex: none;
}

.field-button,
.field-select {
  background: transparent;
  color: inherit;
  font-size: inherit;
}

/* 可交互字段用 pointer，让「可以点」这件事一眼可见 */
.field-button,
.field-select {
  cursor: pointer;
}

.field-button:hover,
.field-select:hover {
  background: var(--pm-brand-soft);
  border-color: var(--pm-brand-border);
  color: var(--pm-brand-strong);
}

.field-select select {
  height: 100%;
  padding: 0;
  border: none;
  background: transparent;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  cursor: pointer;
  /* 去掉系统下拉箭头，保持胶囊的整洁观感 */
  appearance: none;
}

/* 禁用态：无悬停反馈，也不显示手型 */
.field-select.is-disabled,
.field-select.is-disabled:hover {
  background: transparent;
  border-color: transparent;
  color: var(--pm-fg-subtle);
  cursor: default;
}

.field-select.is-disabled select {
  color: var(--pm-fg-subtle);
  cursor: default;
}
</style>
