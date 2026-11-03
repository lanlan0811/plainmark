<script setup lang="ts">
/**
 * 应用根组件：组装标签栏、编辑区与状态栏，并处理来自原生菜单的事件。
 *
 * 所有菜单行为都在这里按 id 分发。原生菜单只在 Rust 侧处理「新建窗口」，
 * 其余一律通过 `plainmark://menu` 事件送到这里。
 */
import { onMounted, onBeforeUnmount, ref, watch, nextTick } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { EditorView } from "@codemirror/view";
import { undo, redo, selectAll, deleteCharBackward } from "@codemirror/commands";
import { openSearchPanel, findNext, findPrevious } from "@codemirror/search";

import TabBar from "./components/TabBar.vue";
import StatusBar from "./components/StatusBar.vue";
import ConfirmSaveModal from "./components/ConfirmSaveModal.vue";
import FontDialog from "./components/FontDialog.vue";
import PreviewPane from "./components/PreviewPane.vue";
import { useDocuments } from "./composables/useDocuments";
import { createEditorState } from "./editor/setup";
import { getState } from "./editor/documents";
import { UNTITLED } from "./api";
import * as api from "./api";
import {
  ZOOM_MAX,
  ZOOM_MIN,
  ZOOM_STEP,
  type Encoding,
  type Eol,
  type MenuEvent,
} from "./types";

const docs = useDocuments();
const editorHost = ref<HTMLElement | null>(null);
let view: EditorView | null = null;

/** 光标位置，供状态栏显示 */
const cursorLine = ref(1);
const cursorColumn = ref(1);

/** 未保存提示的待办动作 */
const pendingAction = ref<null | (() => void | Promise<void>)>(null);
const confirmVisible = ref(false);
const confirmName = ref(UNTITLED);

/** 字体对话框 */
const fontDialogVisible = ref(false);

/** 另存为时的编码选择（简化：用系统保存对话框拿到路径后，按当前编码写入） */
const unlisteners: UnlistenFn[] = [];

// ---------- 预览侧边栏 ----------

/** 预览是否显示。宽度与开关状态持久化到 localStorage。 */
const previewVisible = ref(readBool("plainmark.previewVisible", true));
const previewWidth = ref(readNumber("plainmark.previewWidth", 420, 260, 900));

/** 当前文档源码，供预览渲染。由编辑器内容同步而来。 */
const previewSource = ref("");

function readBool(key: string, fallback: boolean): boolean {
  try {
    const v = localStorage.getItem(key);
    return v === null ? fallback : v === "true";
  } catch {
    return fallback;
  }
}

function readNumber(key: string, fallback: number, min: number, max: number): number {
  try {
    const v = Number(localStorage.getItem(key));
    return Number.isFinite(v) && v >= min && v <= max ? v : fallback;
  } catch {
    return fallback;
  }
}

function persist() {
  try {
    localStorage.setItem("plainmark.previewVisible", String(previewVisible.value));
    localStorage.setItem("plainmark.previewWidth", String(previewWidth.value));
  } catch {
    // localStorage 不可用时忽略，不影响功能
  }
}

watch([previewVisible, previewWidth], persist);

/** 把编辑器内容同步到预览。 */
function syncPreview() {
  const v = view;
  previewSource.value = v ? v.state.doc.toString() : "";
}

/** 分隔条拖拽调宽。 */
let resizing = false;
function onResizeStart(event: MouseEvent) {
  event.preventDefault();
  resizing = true;
  document.body.style.cursor = "col-resize";
  // 拖拽期间禁用文本选中，否则会选中编辑器内容
  document.body.style.userSelect = "none";

  window.addEventListener("mousemove", onResizeMove);
  window.addEventListener("mouseup", onResizeEnd);
}

function onResizeMove(event: MouseEvent) {
  if (!resizing) return;
  // 预览在右侧，宽度 = 窗口右边缘 - 鼠标位置
  const next = window.innerWidth - event.clientX;
  previewWidth.value = Math.min(900, Math.max(260, next));
}

function onResizeEnd() {
  resizing = false;
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  window.removeEventListener("mousemove", onResizeMove);
  window.removeEventListener("mouseup", onResizeEnd);
}

// ---------- 编辑器初始化 ----------

function buildEditor() {
  if (!editorHost.value) return;

  const first = docs.tabs.value[0];
  const state = (first ? getState(first.id) : undefined)
    ?? createEditorState("", docs.wordWrap.value, docs.fontSizeCss.value);

  view = new EditorView({
    state,
    parent: editorHost.value,
    dispatch: (tr) => {
      view!.update([tr]);
      // 内容变化即标记为脏
      if (tr.docChanged) {
        docs.persistActiveState();
        docs.markDirty();
        syncPreview();
      }
      updateCursor();
    },
  });

  docs.attachView(view);
  updateCursor();
  syncPreview();
}

/** 把编辑器内容整体替换掉（拖拽重排、插入元素后使用）。 */
function replaceEditorText(text: string) {
  const v = view;
  if (!v) return;
  v.dispatch({
    changes: { from: 0, to: v.state.doc.length, insert: text },
  });
  v.focus();
}

/** 在光标处插入 Markdown 片段。 */
function insertAtCursor(markdown: string) {
  const v = view;
  if (!v) return;

  const sel = v.state.selection.main;
  // 块级元素前后留空行，避免与相邻内容粘连
  const before = sel.from > 0 ? "\n\n" : "";
  const after = "\n\n";
  const insert = `${before}${markdown}${after}`;

  v.dispatch({
    changes: { from: sel.from, to: sel.to, insert },
    selection: { anchor: sel.from + insert.length },
  });
  v.focus();
}

// ---------- 本地文件拖入 ----------

/**
 * 导入拖入的图片：复制到 assets/ 并插入引用。
 *
 * 路径来自 Tauri 的原生拖放事件——Tauri 在 Windows 上接管了平台拖放，
 * HTML5 的 drop 事件拿不到文件路径。
 */
async function importDroppedFiles(paths: string[]) {
  const tab = docs.activeTab.value;
  if (!tab) return;

  if (!tab.path) {
    window.alert("请先保存文档。图片会存放在文档同目录的 assets/ 下。");
    return;
  }

  const images = paths.filter((p) => /\.(png|jpe?g|gif|webp|bmp|svg|avif|ico)$/i.test(p));
  if (images.length === 0) {
    window.alert("只支持图片文件（png / jpg / gif / webp / bmp / svg 等）。");
    return;
  }

  for (const sourcePath of images) {
    try {
      const asset = await api.importImage(tab.path, sourcePath);
      const alt = asset.name.replace(/\.[^.]+$/, "");
      insertAtCursor(`![${alt}](${asset.relativePath})`);
    } catch (e) {
      window.alert(`导入失败：${e}`);
    }
  }
}

function updateCursor() {
  if (!view) return;
  const sel = view.state.selection.main;
  const line = view.state.doc.lineAt(sel.head);
  cursorLine.value = line.number;
  // CodeMirror 与 Win32 编辑控件一样按 UTF-16 码元计列，Tab 计 1 列
  cursorColumn.value = sel.head - line.from + 1;
}

// ---------- 文件操作 ----------

async function doSave(force = false): Promise<boolean> {
  const tab = docs.activeTab.value;
  if (!tab) return false;

  // 无路径 → 走另存为
  if (!tab.path) return doSaveAs(force);

  const result = await docs.saveActive(force);
  if (!result) return false;

  if (!result.ok && result.lossy) {
    // 编码无法表示部分字符，向用户确认
    const sample = result.lossySample.join(" ");
    const ok = window.confirm(
      `此文件包含无法用 ${tab.encoding} 编码保存的字符：\n${sample}\n\n` +
        `如果按当前编码保存，这些字符会丢失。是否继续？`,
    );
    if (ok) return doSave(true);
    return false;
  }
  return result.ok;
}

async function doSaveAs(force = false): Promise<boolean> {
  const tab = docs.activeTab.value;
  if (!tab) return false;

  const defaultName = tab.path ? tab.name : `${UNTITLED}.md`;
  const path = await api.pickSavePath(defaultName);
  if (!path) return false;

  const result = await docs.saveActiveAs(path, tab.encoding, tab.eol, force);
  if (!result.ok && result.lossy) {
    const sample = result.lossySample.join(" ");
    const ok = window.confirm(
      `此文件包含无法用 ${tab.encoding} 编码保存的字符：\n${sample}\n\n` +
        `如果按当前编码保存，这些字符会丢失。是否继续？`,
    );
    if (ok) return doSaveAs(true);
    return false;
  }
  return result.ok;
}

async function doOpen() {
  const path = await api.pickOpenPath();
  if (!path) return;
  await docs.openPath(path);
}

/**
 * 需要先处理未保存内容的动作，统一走这里。
 * 与记事本一致：保存 / 不保存 / 取消。
 */
function guardUnsaved(action: () => void | Promise<void>) {
  const tab = docs.activeTab.value;
  if (tab?.dirty) {
    confirmName.value = tab.name;
    pendingAction.value = action;
    confirmVisible.value = true;
    return;
  }
  void action();
}

async function onConfirmSave() {
  confirmVisible.value = false;
  const ok = await doSave();
  if (!ok) return;
  const action = pendingAction.value;
  pendingAction.value = null;
  if (action) await action();
}

async function onConfirmDiscard() {
  confirmVisible.value = false;
  const tab = docs.activeTab.value;
  if (tab) tab.dirty = false;
  const action = pendingAction.value;
  pendingAction.value = null;
  if (action) await action();
}

function onConfirmCancel() {
  confirmVisible.value = false;
  pendingAction.value = null;
}

function doNewTab() {
  guardUnsaved(() => {
    docs.newTab();
  });
}

function doCloseTab(id: number) {
  const target = docs.tabs.value.find((t) => t.id === id);
  if (!target) return;
  if (target.dirty) {
    // 关非活动标签时先切过去，让用户看到内容
    docs.activate(id);
    guardUnsaved(() => docs.closeTab(id));
    return;
  }
  docs.closeTab(id);
}

async function doExit() {
  // 「文件 → 退出」关掉整个应用，而不是只关当前窗口
  const quit = () => void api.exitApp();
  const dirty = docs.tabs.value.find((t) => t.dirty);
  if (dirty) {
    docs.activate(dirty.id);
    guardUnsaved(quit);
    return;
  }
  quit();
}

// ---------- 编辑操作 ----------

function withView(fn: (v: EditorView) => void) {
  if (view) {
    fn(view);
    view.focus();
  }
}

function insertTimeDate() {
  withView((v) => {
    const now = new Date();
    const pad = (n: number) => String(n).padStart(2, "0");
    // 记事本格式：HH:mm yyyy/M/d（月日不补零）
    const text =
      `${pad(now.getHours())}:${pad(now.getMinutes())} ` +
      `${now.getFullYear()}/${now.getMonth() + 1}/${now.getDate()}`;
    v.dispatch(v.state.replaceSelection(text));
  });
}

function doGoto() {
  const input = window.prompt("行号：");
  if (!input) return;
  const line = Number.parseInt(input, 10);
  if (!Number.isFinite(line) || line < 1) return;
  withView((v) => {
    // CodeMirror 的 gotoLine 会弹自己的输入框，这里先用 state 把光标移过去，
    // 避免弹两次。用 dispatch 直接定位到目标行首。
    const total = v.state.doc.lines;
    const target = Math.min(line, total);
    const info = v.state.doc.line(target);
    v.dispatch({
      selection: { anchor: info.from },
      scrollIntoView: true,
    });
  });
}

function doFind() {
  withView((v) => openSearchPanel(v));
}

function doFindNext() {
  withView((v) => void findNext(v));
}

function doFindPrev() {
  withView((v) => void findPrevious(v));
}

function doReplace() {
  withView((v) => openSearchPanel(v));
}

// ---------- 缩放 ----------

function zoomIn() {
  docs.zoom.value = Math.min(ZOOM_MAX, docs.zoom.value + ZOOM_STEP);
  docs.applyZoom();
}

function zoomOut() {
  docs.zoom.value = Math.max(ZOOM_MIN, docs.zoom.value - ZOOM_STEP);
  docs.applyZoom();
}

function zoomReset() {
  docs.zoom.value = 100;
  docs.applyZoom();
}

// ---------- 菜单事件分发 ----------

async function handleMenu(event: MenuEvent) {
  switch (event.id) {
    case "file.new":
      doNewTab();
      break;
    case "file.open":
      guardUnsaved(doOpen);
      break;
    case "file.save":
      await doSave();
      break;
    case "file.save_as":
      await doSaveAs();
      break;
    case "file.exit":
      await doExit();
      break;

    case "edit.undo":
      withView((v) => undo(v));
      break;
    case "edit.redo":
      withView((v) => redo(v));
      break;
    case "edit.cut":
      document.execCommand("cut");
      break;
    case "edit.copy":
      document.execCommand("copy");
      break;
    case "edit.paste":
      void navigator.clipboard.readText().then((text) => {
        withView((v) => v.dispatch(v.state.replaceSelection(text)));
      });
      break;
    case "edit.delete":
      withView((v) => deleteCharBackward(v));
      break;
    case "edit.find":
      doFind();
      break;
    case "edit.find_next":
      doFindNext();
      break;
    case "edit.find_prev":
      doFindPrev();
      break;
    case "edit.replace":
      doReplace();
      break;
    case "edit.goto":
      doGoto();
      break;
    case "edit.select_all":
      withView((v) => selectAll(v));
      break;
    case "edit.time_date":
      insertTimeDate();
      break;

    case "format.wordwrap":
      docs.wordWrap.value = !docs.wordWrap.value;
      docs.applyWordWrap();
      break;
    case "format.font":
      fontDialogVisible.value = true;
      break;

    case "view.zoom_in":
      zoomIn();
      break;
    case "view.zoom_out":
      zoomOut();
      break;
    case "view.zoom_reset":
      zoomReset();
      break;
    case "view.statusbar":
      docs.statusBarVisible.value = !docs.statusBarVisible.value;
      break;
    case "view.preview":
      previewVisible.value = !previewVisible.value;
      break;

    case "help.view":
      await api.openHelp();
      break;
    case "help.about":
      window.alert(
        `plainmark 0.1.0\n\n像记事本一样简单的 Markdown 编辑器\n\n` +
          `MIT License © 2026 lanlan0811`,
      );
      break;
    default:
      break;
  }
}

// ---------- 状态栏交互 ----------

function onSetEol(eol: Eol) {
  docs.setActiveEol(eol);
}

function onSetEncoding(encoding: Encoding) {
  const tab = docs.activeTab.value;
  if (!tab || !tab.path) return;
  if (tab.encoding === encoding) return;
  // 编码变更需重新写盘才能生效
  tab.encoding = encoding;
  tab.dirty = true;
  docs.syncTitle();
}

function onSetZoom(zoom: number) {
  docs.zoom.value = zoom;
  docs.applyZoom();
}

function onFontApply(value: {
  family: string;
  sizePt: number;
  bold: boolean;
  italic: boolean;
}) {
  docs.fontFamily.value = value.family;
  docs.fontSizePt.value = value.sizePt;
  docs.fontBold.value = value.bold;
  docs.fontItalic.value = value.italic;
  fontDialogVisible.value = false;
  document.documentElement.style.setProperty("--pm-font-family", value.family);
  docs.applyZoom();
}

// ---------- 生命周期 ----------

onMounted(async () => {
  await nextTick();
  docs.newTab();
  buildEditor();

  // 菜单事件
  unlisteners.push(await listen<MenuEvent>("plainmark://menu", (e) => void handleMenu(e.payload)));

  // 第二次启动时由单实例回调送来的文件
  unlisteners.push(
    await listen<string[]>("plainmark://open-files", async (e) => {
      for (const path of e.payload) await docs.openPath(path);
    }),
  );

  // 启动时命令行传入的文件
  const startup = await api.getStartupFiles();
  for (const path of startup) await docs.openPath(path);

  // 拖放打开文件
  unlisteners.push(
    await getCurrentWindow().onDragDropEvent(async (e) => {
      if (e.payload.type !== "drop") return;

      const paths = e.payload.paths;

      // 图片 → 导入到 assets/ 并插入引用
      if (paths.some((p) => /\.(png|jpe?g|gif|webp|bmp|svg|avif|ico)$/i.test(p))) {
        await importDroppedFiles(paths);
        return;
      }

      // 文档 → 打开成新标签
      for (const path of paths) {
        if (/\.(md|markdown|txt)$/i.test(path)) await docs.openPath(path);
      }
    }),
  );

  // 关闭窗口前拦截未保存内容
  unlisteners.push(
    await getCurrentWindow().onCloseRequested(async (e) => {
      const dirty = docs.tabs.value.find((t) => t.dirty);
      if (dirty) {
        e.preventDefault();
        docs.activate(dirty.id);
        guardUnsaved(() => void api.closeWindow());
      }
    }),
  );

  // 窗口重获焦点时检查文件是否被外部改动，与记事本行为一致
  unlisteners.push(
    await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) void onWindowFocus();
    }),
  );
});

onBeforeUnmount(() => {
  window.clearTimeout(focusTimer);
  for (const un of unlisteners) un();
  view?.destroy();
});

/**
 * 外部改动检测。
 *
 * 记事本在窗口重新获得焦点时比对文件的修改时间，若已变化则询问是否重新加载。
 * 这里采用同样的策略——比轮询省资源，也符合用户的心理模型。
 */
async function checkExternalChanges() {
  for (const tab of docs.tabs.value) {
    if (!tab.path) continue;

    const current = await api.fileModifiedMs(tab.path);

    if (current === null) {
      // 文件已被删除或移动
      if (!tab.reloadPending) {
        tab.reloadPending = true;
        window.alert(`${tab.name} 已被删除或移动。`);
      }
      continue;
    }

    if (tab.modifiedMs !== null && current !== tab.modifiedMs) {
      const reload = window.confirm(
        `${tab.name} 已被其他程序修改。\n\n是否要重新加载？`,
      );
      if (reload) {
        docs.activate(tab.id);
        await docs.reloadActive();
      } else {
        // 用户选择不重新加载：记下已看到的这一版，这样文件再次变动时仍会提示，
        // 而不是因为记住「已拒绝」就永远不再询问。
        tab.modifiedMs = current;
      }
    }
  }
}

let focusTimer: number | undefined;
async function onWindowFocus() {
  // 焦点事件可能密集触发，去抖后再检查
  window.clearTimeout(focusTimer);
  focusTimer = window.setTimeout(() => void checkExternalChanges(), 300);
}

// 脏状态变化时通知后端
watch(
  () => docs.anyDirty.value,
  (dirty) => void api.setWindowDirty(dirty),
);
</script>

<template>
  <div class="app">
    <TabBar
      :tabs="docs.tabs.value"
      :active-id="docs.activeId.value"
      @select="docs.activate"
      @close="doCloseTab"
      @new="doNewTab"
    />

    <!-- 工作区：编辑区 + 预览侧边栏 -->
    <div class="workspace">
      <div ref="editorHost" class="editor-host"></div>

      <!-- 分隔条：拖动调宽 -->
      <div
        v-if="previewVisible"
        class="splitter"
        role="separator"
        aria-orientation="vertical"
        title="拖动调整宽度"
        @mousedown="onResizeStart"
      ></div>

      <div
        v-if="previewVisible"
        class="preview-side"
        :style="{ width: previewWidth + 'px' }"
      >
        <PreviewPane
          :source="previewSource"
          :doc-path="docs.activeTab.value?.path ?? null"
          @reorder="replaceEditorText"
        />
      </div>
    </div>

    <StatusBar
      v-if="docs.statusBarVisible.value"
      :line="cursorLine"
      :column="cursorColumn"
      :zoom="docs.zoom.value"
      :eol="docs.activeTab.value?.eol ?? 'Crlf'"
      :encoding="docs.activeTab.value?.encoding ?? 'Utf8'"
      :has-path="!!docs.activeTab.value?.path"
      @set-eol="onSetEol"
      @set-encoding="onSetEncoding"
      @set-zoom="onSetZoom"
    />

    <ConfirmSaveModal
      :visible="confirmVisible"
      :name="confirmName"
      @save="onConfirmSave"
      @discard="onConfirmDiscard"
      @cancel="onConfirmCancel"
    />

    <FontDialog
      :visible="fontDialogVisible"
      :family="docs.fontFamily.value"
      :size-pt="docs.fontSizePt.value"
      :bold="docs.fontBold.value"
      :italic="docs.fontItalic.value"
      @apply="onFontApply"
      @cancel="fontDialogVisible = false"
    />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  /* 外圈浅色 chrome，编辑区与预览是内嵌的白色面板——形成明确的层次感 */
  background: var(--pm-chrome);
}

.workspace {
  display: flex;
  flex: 1;
  min-height: 0;
  padding: 6px 8px;
  gap: 0;
}

.editor-host {
  flex: 1;
  min-width: 0;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-sm);
  overflow: hidden;
}

/* 分隔条：细，但留出足够的抓取宽度 */
.splitter {
  flex: none;
  width: 8px;
  cursor: col-resize;
  position: relative;
}

.splitter::after {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 2px;
  height: 32px;
  border-radius: 999px;
  background: var(--pm-border-strong);
  transition: background var(--pm-transition);
}

.splitter:hover::after {
  background: var(--pm-brand);
}

.preview-side {
  flex: none;
  display: flex;
  flex-direction: column;
  min-width: 0;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-sm);
  overflow: hidden;
}
</style>
