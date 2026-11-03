<script setup lang="ts">
/**
 * 元素面板：可拖入预览的素材模板。
 *
 * 拖拽走 pointer 事件而非 HTML5 拖放——Tauri 在 Windows 上接管了 WebView2 的
 * 拖放处理器，HTML5 的 drag 事件不会触发（详见 usePointerDrag.ts）。
 *
 * 也支持点击插入：拖拽在触控板上不好用，给一条退路。
 */
import { ref } from "vue";
import { startDrag } from "../composables/usePointerDrag";

const emit = defineEmits<{
  /** 点击插入：把 Markdown 片段追加到光标处 */
  (e: "insert", markdown: string): void;
  /** 拖拽开始，父组件据此在预览上显示落点 */
  (e: "drag-start", markdown: string): void;
  /** 拖拽移动（视口坐标） */
  (e: "drag-move", x: number, y: number): void;
  /** 拖拽结束（视口坐标） */
  (e: "drag-end", x: number, y: number): void;
  /** 拖拽取消 */
  (e: "drag-cancel"): void;
}>();

interface ElementTemplate {
  id: string;
  label: string;
  /** 插入到 Markdown 源码里的片段 */
  markdown: string;
  /** 面板上的预览字符 */
  glyph: string;
}

/** 常用素材模板。徽章走 shields.io，图标走内联 SVG 便于离线使用。 */
const TEMPLATES: ElementTemplate[] = [
  {
    id: "badge-license",
    label: "MIT 许可徽章",
    markdown: "![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)",
    glyph: "MIT",
  },
  {
    id: "badge-platform",
    label: "平台徽章",
    markdown: "![Platform: Windows](https://img.shields.io/badge/Platform-Windows%2010%2B-blue.svg)",
    glyph: "Win",
  },
  {
    id: "badge-build",
    label: "构建状态徽章",
    markdown: "![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)",
    glyph: "CI",
  },
  {
    id: "badge-version",
    label: "版本徽章",
    markdown: "![Version](https://img.shields.io/badge/version-0.1.0-informational.svg)",
    glyph: "v",
  },
  {
    id: "rule",
    label: "分隔线",
    markdown: "---",
    glyph: "—",
  },
  {
    id: "quote",
    label: "引用块",
    markdown: "> 在此输入引用内容",
    glyph: "❝",
  },
  {
    id: "code",
    label: "代码块",
    markdown: "```\n在此输入代码\n```",
    glyph: "{ }",
  },
  {
    id: "table",
    label: "三列表格",
    markdown: "| 列 1 | 列 2 | 列 3 |\n| --- | --- | --- |\n| 内容 | 内容 | 内容 |",
    glyph: "▦",
  },
  {
    id: "task",
    label: "任务列表",
    markdown: "- [ ] 待办事项\n- [x] 已完成事项",
    glyph: "☑",
  },
  {
    id: "image",
    label: "图片占位",
    markdown: "![描述](./assets/图片.png)",
    glyph: "🖼",
  },
];

/** 拖拽中的模板 id，用于高亮。 */
const draggingId = ref<string | null>(null);

/** 按下位置，移动超过阈值才认定为拖拽。 */
let pressX = 0;
let pressY = 0;

function onPointerDown(event: PointerEvent, tpl: ElementTemplate) {
  if (event.button !== 0) return;

  pressX = event.clientX;
  pressY = event.clientY;
  let started = false;

  startDrag(
    { kind: "element", markdown: tpl.markdown, label: tpl.label },
    {
      onMove: (x, y) => {
        if (!started) {
          if (Math.hypot(x - pressX, y - pressY) < 5) return;
          started = true;
          draggingId.value = tpl.id;
          emit("drag-start", tpl.markdown);
        }
        emit("drag-move", x, y);
      },
      onDrop: (x, y) => {
        // 没移动过就当作点击，交给 click 事件处理
        if (started) {
          emit("drag-end", x, y);
        }
        draggingId.value = null;
      },
      onCancel: () => {
        draggingId.value = null;
        emit("drag-cancel");
      },
    },
  );
}
</script>

<template>
  <div class="panel">
    <p class="panel-title">元素</p>
    <p class="panel-hint">拖到右侧预览，或点击插入</p>

    <div class="grid">
      <button
        v-for="tpl in TEMPLATES"
        :key="tpl.id"
        class="item"
        :class="{ dragging: draggingId === tpl.id }"
        type="button"
        :title="`${tpl.label}（拖到预览，或点击插入）`"
        @pointerdown="onPointerDown($event, tpl)"
        @click="emit('insert', tpl.markdown)"
      >
        <span class="glyph">{{ tpl.glyph }}</span>
        <span class="label">{{ tpl.label }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 12px;
  background: var(--pm-chrome);
  overflow-y: auto;
}

.panel-title {
  margin: 0 0 2px;
  font-size: 12px;
  font-weight: 600;
  color: var(--pm-fg);
}

.panel-hint {
  margin: 0 0 12px;
  font-size: 11px;
  color: var(--pm-fg-subtle);
}

.grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 72px;
  padding: 10px 6px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-bg);
  color: var(--pm-fg-muted);
  transition:
    border-color var(--pm-transition),
    background var(--pm-transition),
    color var(--pm-transition),
    transform var(--pm-transition);
}

.item:hover {
  border-color: var(--pm-brand-border);
  background: var(--pm-brand-soft);
  color: var(--pm-brand-strong);
}

.item:active {
  transform: scale(0.97);
}

.item.dragging {
  opacity: 0.45;
}

.glyph {
  font-size: 15px;
  font-weight: 600;
  line-height: 1;
  color: var(--pm-brand);
}

.label {
  font-size: 10px;
  text-align: center;
  line-height: 1.3;
  color: var(--pm-fg-subtle);
}
</style>
