<script setup lang="ts">
/**
 * Markdown 可视化预览侧边栏。
 *
 * 三条硬性约束（按需求）：
 *   1. **只读**：预览里的文字不能编辑。
 *   2. **可拖拽重排**：拖动预览中的块改变顺序，同步改写 Markdown 源码。
 *   3. **可调宽、可开关**：宽度由父组件控制，本组件只负责渲染与拖拽交互。
 *
 * 实现要点：
 *   - 内容是 v-html 插入的，Vue 管不到这些节点，渲染后需要一次 DOM 后处理。
 *   - 拖拽走 pointer 事件而非 HTML5 拖放：Tauri 在 Windows 上默认接管了
 *     WebView2 的拖放处理器，HTML5 的 drag 事件不会触发。详见 usePointerDrag.ts。
 *   - 事件用委托挂在容器上，不给每个块单独绑定。
 */
import { ref, watch, nextTick } from "vue";
import { renderMarkdown, extractBlocks, moveBlockTo, type SourceBlock } from "../preview/renderer";
import { startDrag, isPointInside } from "../composables/usePointerDrag";

const props = defineProps<{
  /** 当前文档源码 */
  source: string;
  /** 文档路径，仅用于在文档切换时重新渲染 */
  docPath: string | null;
}>();

const emit = defineEmits<{
  /** 重排后要把新源码写回编辑器 */
  (e: "reorder", text: string): void;
}>();

const host = ref<HTMLElement | null>(null);
const html = ref("");
const error = ref<string | null>(null);

/** 渲染后处理：标记任务列表项。 */
function decorate() {
  const el = host.value;
  if (!el) return;

  // 任务列表项去掉项目符号。
  // 用 DOM 标记而非 CSS :has()——:has() 从 Chromium 105 才默认启用，
  // 而构建目标正是 105，踩在边界上不如显式标记稳妥。
  el.querySelectorAll<HTMLLIElement>("li").forEach((li) => {
    if (li.querySelector(':scope > input[type="checkbox"]')) {
      li.classList.add("pm-task-item");
    }
  });
}

let renderTimer: number | undefined;
function scheduleRender() {
  window.clearTimeout(renderTimer);
  renderTimer = window.setTimeout(async () => {
    try {
      html.value = renderMarkdown(props.source).html;
      error.value = null;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      html.value = "";
    }
    await nextTick();
    decorate();
  }, 120);
}

watch(() => props.source, scheduleRender, { immediate: true });
watch(() => props.docPath, scheduleRender);

/** 取当前渲染出的块区间。 */
function currentBlocks(): SourceBlock[] {
  return host.value ? extractBlocks(host.value) : [];
}

/** 所有块的 DOM 节点，按文档顺序。 */
function blockNodes(): HTMLElement[] {
  if (!host.value) return [];
  return Array.from(host.value.querySelectorAll<HTMLElement>(".pm-block"));
}

/** 根据指针坐标找出落点所在的块下标。 */
function blockIndexAt(x: number, y: number): number {
  const nodes = blockNodes();
  for (let i = 0; i < nodes.length; i++) {
    if (isPointInside(nodes[i]!, x, y)) return i;
  }
  return -1;
}

function clearMarks() {
  const el = host.value;
  if (!el) return;
  el.querySelectorAll(".pm-dragging, .pm-drop-target").forEach((n) => {
    n.classList.remove("pm-dragging", "pm-drop-target");
  });
}

// ---------- 预览内：块重排 ----------

function onPointerDown(event: PointerEvent) {
  // 只响应左键，且不在预览的滚动条上启动
  if (event.button !== 0) return;

  const target = event.target;
  if (!(target instanceof HTMLElement)) return;

  const block = target.closest<HTMLElement>(".pm-block");
  if (!block) return;

  const nodes = blockNodes();
  const from = nodes.indexOf(block);
  if (from < 0) return;

  // 记录按下位置，移动超过阈值才认定为拖拽——否则点击链接会被误判
  const startX = event.clientX;
  const startY = event.clientY;
  let started = false;

  const onMove = (x: number, y: number) => {
    if (!started) {
      if (Math.hypot(x - startX, y - startY) < 5) return;
      started = true;
      block.classList.add("pm-dragging");
    }

    // 高亮落点
    const to = blockIndexAt(x, y);
    const el = host.value;
    el?.querySelectorAll(".pm-drop-target").forEach((n) =>
      n.classList.remove("pm-drop-target"),
    );
    if (to >= 0 && to !== from) nodes[to]?.classList.add("pm-drop-target");
  };

  const onDrop = (x: number, y: number) => {
    if (!started) return;
    const to = blockIndexAt(x, y);
    const blocks = currentBlocks();

    if (to >= 0 && to !== from && from < blocks.length && to < blocks.length) {
      const next = moveBlockTo(props.source, blocks, from, to);
      if (next !== props.source) emit("reorder", next);
    }
    clearMarks();
  };

  startDrag(
    { kind: "block", index: from },
    { onMove, onDrop, onCancel: clearMarks },
  );

  // 阻止链接、图片的默认拖拽行为
  event.preventDefault();
}

</script>

<template>
  <div class="preview">
    <div v-if="error" class="error">
      <p class="error-title">预览渲染失败</p>
      <p class="error-detail">{{ error }}</p>
    </div>

    <div v-else-if="!source.trim()" class="empty">
      <p>文档为空</p>
      <p class="hint">开始输入，这里会实时呈现渲染结果</p>
    </div>

    <!--
      内容经 DOMPurify 清洗后插入。整体不可编辑；
      按下并拖动块可重排，会通过 reorder 事件把新源码写回编辑器。
      拖拽用 pointer 事件：Tauri 接管了平台拖放，HTML5 drag 事件不会触发。
    -->
    <div
      v-else
      ref="host"
      class="content pm-preview-content"
      v-html="html"
      @pointerdown="onPointerDown"
    ></div>
  </div>
</template>

<style scoped>
.preview {
  height: 100%;
  overflow: auto;
  background: var(--pm-bg);
  /* 预览整体不可选中，明确「这里不能编辑文字」 */
  user-select: none;
}

.content {
  padding: 16px 20px 32px;
  color: var(--pm-fg);
  font-size: 13px;
  line-height: 1.7;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 100%;
  color: var(--pm-fg-subtle);
  font-size: 12px;
}

.empty .hint {
  font-size: 11px;
  opacity: 0.8;
}

.error {
  margin: 16px;
  padding: 12px 14px;
  border: 1px solid #f0c2c2;
  border-radius: var(--pm-radius-md);
  background: #fff5f5;
  color: #a33;
  font-size: 12px;
}

.error-title {
  margin: 0 0 4px;
  font-weight: 600;
}

.error-detail {
  margin: 0;
  word-break: break-word;
  opacity: 0.85;
}
</style>

<!--
  预览内容的样式不能加 scoped —— v-html 插入的节点不带组件的 scope 属性，
  scoped 选择器匹配不到它们。
-->
<style>
.pm-preview-content {
  word-wrap: break-word;
}

/* ---- 块：拖拽重排的单元 ---- */
.pm-preview-content .pm-block {
  position: relative;
  border-radius: var(--pm-radius-sm);
  transition:
    background var(--pm-transition),
    box-shadow var(--pm-transition);
}

/* 悬停暗示「可拖动」。用 box-shadow 模拟外扩底色，不影响布局。 */
.pm-preview-content .pm-block:hover {
  background: var(--pm-chrome);
  box-shadow: 0 0 0 6px var(--pm-chrome);
}

/* 拖动中的源块 */
.pm-preview-content .pm-block.pm-dragging {
  opacity: 0.35;
}

/* 放置目标：顶部品牌色指示线 */
.pm-preview-content .pm-block.pm-drop-target {
  box-shadow:
    0 -2px 0 0 var(--pm-brand),
    0 0 0 6px var(--pm-brand-soft);
}

/* 预览内容默认不可选中；拖拽时给出抓取光标 */
.pm-preview-content .pm-block {
  cursor: default;
}

.pm-preview-content .pm-block:hover {
  cursor: grab;
}

.pm-preview-content .pm-block:active {
  cursor: grabbing;
}

/* 链接在拖拽容器内不应触发浏览器原生拖拽 */
.pm-preview-content a,
.pm-preview-content img {
  -webkit-user-drag: none;
}

/* ---- 标题 ---- */
.pm-preview-content h1,
.pm-preview-content h2,
.pm-preview-content h3,
.pm-preview-content h4,
.pm-preview-content h5,
.pm-preview-content h6 {
  margin: 1.4em 0 0.6em;
  font-weight: 600;
  line-height: 1.35;
}

.pm-preview-content h1 {
  font-size: 1.7em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--pm-border);
}

.pm-preview-content h2 {
  font-size: 1.35em;
  padding-bottom: 0.25em;
  border-bottom: 1px solid var(--pm-border);
}

.pm-preview-content h3 {
  font-size: 1.15em;
}

.pm-preview-content h4 {
  font-size: 1em;
}

/* 首块不留过大的上边距 */
.pm-preview-content > .pm-block:first-child > h1,
.pm-preview-content > .pm-block:first-child > h2,
.pm-preview-content > .pm-block:first-child > h3 {
  margin-top: 0;
}

/* ---- 正文 ---- */
.pm-preview-content p {
  margin: 0.7em 0;
}

.pm-preview-content a {
  color: var(--pm-brand-strong);
  text-decoration: none;
  border-bottom: 1px solid var(--pm-brand-border);
}

.pm-preview-content a:hover {
  border-bottom-color: var(--pm-brand);
}

.pm-preview-content strong {
  font-weight: 600;
}

/* ---- 列表 ---- */
.pm-preview-content ul,
.pm-preview-content ol {
  margin: 0.7em 0;
  padding-left: 1.6em;
}

.pm-preview-content li {
  margin: 0.25em 0;
}

/* 任务列表项：标记类由 decorate() 在渲染后加上，不依赖 :has() */
.pm-preview-content li.pm-task-item {
  list-style: none;
  margin-left: -1.2em;
}

.pm-preview-content input[type="checkbox"] {
  margin-right: 0.4em;
  vertical-align: middle;
}

/* ---- 引用 ---- */
.pm-preview-content blockquote {
  margin: 0.9em 0;
  padding: 0.2em 0 0.2em 1em;
  border-left: 3px solid var(--pm-brand-border);
  color: var(--pm-fg-muted);
}

.pm-preview-content blockquote > :first-child {
  margin-top: 0;
}

.pm-preview-content blockquote > :last-child {
  margin-bottom: 0;
}

/* ---- 代码 ---- */
.pm-preview-content code {
  padding: 0.15em 0.4em;
  border-radius: var(--pm-radius-sm);
  background: var(--pm-chrome-strong);
  font-family: var(--pm-font-family);
  font-size: 0.9em;
  color: #a31515;
}

.pm-preview-content pre {
  margin: 0.9em 0;
  padding: 12px 14px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-chrome);
  overflow-x: auto;
}

.pm-preview-content pre code {
  padding: 0;
  background: transparent;
  color: var(--pm-fg);
  font-size: 0.88em;
}

/* ---- 表格 ---- */
.pm-preview-content table {
  margin: 0.9em 0;
  border-collapse: collapse;
  width: 100%;
  font-size: 0.95em;
}

.pm-preview-content th,
.pm-preview-content td {
  padding: 7px 10px;
  border: 1px solid var(--pm-border);
  text-align: left;
}

.pm-preview-content th {
  background: var(--pm-chrome);
  font-weight: 600;
}

.pm-preview-content tr:nth-child(even) td {
  background: var(--pm-chrome);
}

/* ---- 分隔线 ---- */
.pm-preview-content hr {
  margin: 1.5em 0;
  border: none;
  border-top: 1px solid var(--pm-border);
}

/* ---- 图片与徽章：可拖拽的素材 ---- */
.pm-preview-content img {
  max-width: 100%;
  height: auto;
  border-radius: var(--pm-radius-md);
  cursor: grab;
  transition: box-shadow var(--pm-transition);
}

.pm-preview-content img:hover {
  box-shadow: 0 0 0 3px var(--pm-brand-soft);
}

/* 徽章常并排出现，给一点间距 */
.pm-preview-content p > img + img {
  margin-left: 6px;
}
</style>
