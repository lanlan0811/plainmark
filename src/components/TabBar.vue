<script setup lang="ts">
/**
 * 标签栏。外观对齐 Win11 记事本：细长标签 + 末尾的「+」按钮。
 */
import type { Tab } from "../editor/documents";
import { tabLabel } from "../editor/documents";

const props = defineProps<{
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

/** 中键关闭标签，与主流编辑器一致。 */
function onMouseDown(event: MouseEvent, id: number) {
  if (event.button === 1) {
    event.preventDefault();
    emit("close", id);
  }
}

defineExpose({});
void props;
</script>

<template>
  <div class="tabbar">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === activeId }"
      :title="tab.path ?? tab.name"
      @click="emit('select', tab.id)"
      @mousedown="onMouseDown($event, tab.id)"
    >
      <span class="tab-title">{{ tabLabel(tab) }}</span>
      <button
        class="tab-close"
        type="button"
        title="关闭"
        @click="onClose($event, tab.id)"
      >
        ×
      </button>
    </div>

    <button class="tab-new" type="button" title="新建标签" @click="emit('new')">
      +
    </button>
  </div>
</template>

<style scoped>
.tabbar {
  display: flex;
  align-items: stretch;
  height: 28px;
  background: var(--pm-tabbar-bg);
  border-bottom: 1px solid var(--pm-border);
  overflow-x: auto;
  overflow-y: hidden;
  /* 标签栏不需要滚动条视觉噪音 */
  scrollbar-width: thin;
}

.tab {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 90px;
  max-width: 200px;
  padding: 0 6px 0 10px;
  border-right: 1px solid var(--pm-border);
  cursor: default;
  white-space: nowrap;
  user-select: none;
  color: #444;
}

.tab:hover {
  background: var(--pm-tab-hover-bg);
}

.tab.active {
  background: var(--pm-tab-active-bg);
  /* 顶部的强调线，与 Win11 记事本一致 */
  box-shadow: inset 0 2px 0 #0067c0;
  color: #000;
}

.tab-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 12px;
}

.tab-close {
  flex: none;
  width: 16px;
  height: 16px;
  padding: 0;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: #666;
  font-size: 14px;
  line-height: 14px;
  cursor: default;
}

.tab-close:hover {
  background: #d0d0d0;
  color: #000;
}

.tab-new {
  flex: none;
  width: 28px;
  border: none;
  background: transparent;
  color: #333;
  font-size: 15px;
  cursor: default;
}

.tab-new:hover {
  background: var(--pm-tab-hover-bg);
}
</style>
