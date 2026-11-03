<script setup lang="ts">
/**
 * 标签栏。
 *
 * 视觉上是独立设计：圆角标签、品牌色选中态、悬停反馈。
 * 刻意不做成 Win11 记事本那种「顶部细线 + 全直角」的样式。
 */
import type { Tab } from "../editor/documents";
import { tabLabel } from "../editor/documents";

defineProps<{
  tabs: Tab[];
  activeId: number | null;
}>();

const emit = defineEmits<{
  (e: "select", id: number): void;
  (e: "close", id: number): void;
  (e: "new"): void;
}>();

function onClose(event: MouseEvent, id: number) {
  // 避免冒泡触发选中
  event.stopPropagation();
  emit("close", id);
}

/** 中键关闭标签。 */
function onMouseDown(event: MouseEvent, id: number) {
  if (event.button === 1) {
    event.preventDefault();
    emit("close", id);
  }
}
</script>

<template>
  <div class="tabbar">
    <div class="tab-list">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab"
        :class="{ active: tab.id === activeId }"
        type="button"
        :title="tab.path ?? tab.name"
        @click="emit('select', tab.id)"
        @mousedown="onMouseDown($event, tab.id)"
      >
        <span class="tab-title">{{ tabLabel(tab) }}</span>
        <span
          class="tab-close"
          role="button"
          tabindex="-1"
          title="关闭"
          @click="onClose($event, tab.id)"
        >
          <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
            <path
              d="M1.5 1.5 L8.5 8.5 M8.5 1.5 L1.5 8.5"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
        </span>
      </button>
    </div>

    <button class="tab-new" type="button" title="新建标签" @click="emit('new')">
      <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden="true">
        <path
          d="M6 2 V10 M2 6 H10"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.tabbar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: var(--pm-tabbar-height);
  padding: 0 8px;
  background: var(--pm-chrome);
  border-bottom: 1px solid var(--pm-border);
  flex: none;
}

.tab-list {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
  /* 标签本身已够紧凑，不需要滚动条视觉干扰 */
  scrollbar-width: none;
}

.tab-list::-webkit-scrollbar {
  display: none;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: none;
  min-width: 96px;
  max-width: 210px;
  height: 26px;
  padding: 0 6px 0 12px;
  border: 1px solid transparent;
  border-radius: var(--pm-radius-md);
  background: transparent;
  color: var(--pm-fg-muted);
  font-size: 12px;
  white-space: nowrap;
  transition:
    background var(--pm-transition),
    color var(--pm-transition),
    border-color var(--pm-transition);
}

.tab:hover {
  background: var(--pm-chrome-strong);
  color: var(--pm-fg);
}

/* 选中态：白底 + 品牌色描边，形成明确的「当前文档」标识 */
.tab.active {
  background: var(--pm-bg);
  border-color: var(--pm-brand-border);
  color: var(--pm-fg);
  box-shadow: var(--pm-shadow-sm);
}

.tab-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 18px;
  height: 18px;
  border-radius: var(--pm-radius-sm);
  color: var(--pm-fg-subtle);
  opacity: 0;
  transition:
    opacity var(--pm-transition),
    background var(--pm-transition),
    color var(--pm-transition);
}

/* 关闭按钮平时隐藏，悬停标签或选中时才出现，减少视觉噪音 */
.tab:hover .tab-close,
.tab.active .tab-close {
  opacity: 1;
}

.tab-close:hover {
  background: var(--pm-chrome-strong);
  color: var(--pm-fg);
}

.tab-new {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 26px;
  height: 26px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: var(--pm-radius-md);
  background: transparent;
  color: var(--pm-fg-muted);
  transition:
    background var(--pm-transition),
    color var(--pm-transition),
    border-color var(--pm-transition);
}

.tab-new:hover {
  background: var(--pm-brand-soft);
  border-color: var(--pm-brand-border);
  color: var(--pm-brand-strong);
}
</style>
