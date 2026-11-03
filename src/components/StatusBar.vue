<script setup lang="ts">
/**
 * 状态栏。四个字段对齐记事本：
 *   第 N 行, 第 M 列 | 缩放% | 换行符 | 编码
 *
 * 换行符与编码在 Win11 记事本里是可点击下拉，这里同样做成下拉。
 */
import { ENCODING_LABELS, EOL_CHOICES, EOL_LABELS, type Encoding, type Eol } from "../types";

const props = defineProps<{
  line: number;
  column: number;
  zoom: number;
  eol: Eol;
  encoding: Encoding;
  /** 显示下拉需要区分「无路径的新文档」 */
  hasPath: boolean;
}>();

const emit = defineEmits<{
  (e: "set-eol", eol: Eol): void;
  (e: "set-encoding", encoding: Encoding): void;
  (e: "set-zoom", zoom: number): void;
}>();

function onEolChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as Eol;
  emit("set-eol", value);
}
</script>

<template>
  <div class="statusbar">
    <div class="cell position">第 {{ line }} 行, 第 {{ column }} 列</div>
    <div class="spacer"></div>

    <div class="cell">
      <button class="zoom" type="button" :title="`当前缩放 ${zoom}%`" @click="emit('set-zoom', 100)">
        {{ zoom }}%
      </button>
    </div>

    <div class="cell select-cell">
      <select
        class="field"
        :value="eol"
        title="换行符"
        @change="onEolChange"
      >
        <option v-for="choice in EOL_CHOICES" :key="choice" :value="choice">
          {{ EOL_LABELS[choice] }}
        </option>
      </select>
    </div>

    <div class="cell select-cell">
      <select
        class="field"
        :value="encoding"
        title="编码"
        :disabled="!hasPath"
        @change="emit('set-encoding', ($event.target as HTMLSelectElement).value as Encoding)"
      >
        <option v-for="(label, key) in ENCODING_LABELS" :key="key" :value="key">
          {{ label }}
        </option>
      </select>
    </div>
  </div>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: stretch;
  height: 22px;
  background: var(--pm-statusbar-bg);
  border-top: 1px solid var(--pm-border);
  font-size: 12px;
  color: #333;
  user-select: none;
}

.spacer {
  flex: 1;
}

.cell {
  display: flex;
  align-items: center;
  padding: 0 10px;
  border-left: 1px solid var(--pm-border);
  white-space: nowrap;
}

.cell.position {
  border-left: none;
  min-width: 140px;
}

.select-cell {
  padding: 0;
}

.field {
  height: 100%;
  padding: 0 6px;
  border: none;
  background: transparent;
  font-family: inherit;
  font-size: 12px;
  color: inherit;
  cursor: default;
}

.field:hover:not(:disabled) {
  background: #e0e0e0;
}

.field:disabled {
  color: #999;
}

.zoom {
  border: none;
  background: transparent;
  font-family: inherit;
  font-size: 12px;
  color: inherit;
  cursor: default;
}

.zoom:hover {
  background: #e0e0e0;
}
</style>
