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
      }
      updateCursor();
    },
  });

  docs.attachView(view);
  updateCursor();
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
  const closeWindow = () => void api.closeWindow();
  const dirty = docs.tabs.value.find((t) => t.dirty);
  if (dirty) {
    docs.activate(dirty.id);
    guardUnsaved(closeWindow);
    return;
  }
  closeWindow();
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
      if (e.payload.type === "drop") {
        for (const path of e.payload.paths) {
          if (/\.(md|markdown|txt)$/i.test(path)) await docs.openPath(path);
        }
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
});

onBeforeUnmount(() => {
  for (const un of unlisteners) un();
  view?.destroy();
});

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

    <div ref="editorHost" class="editor-host"></div>

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
}

.editor-host {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
</style>
