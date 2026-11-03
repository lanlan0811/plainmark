/**
 * 标签页与文档状态的集中管理。
 *
 * 所有标签共用一个 CodeMirror EditorView，切换标签时把 view 的 state 换成
 * 该标签保存的那份。EditorState 是不可变的，因此每个标签的撤销历史天然隔离。
 *
 * 注意：EditorState 存放在 editor/documents.ts 的非响应式 Map 里，不能放进
 * Vue 的 ref —— 深度解包会破坏它的私有字段类型。
 */

import { computed, ref, shallowRef } from "vue";
import type { EditorView } from "@codemirror/view";
import {
  createTab,
  dropState,
  getState,
  setState,
  tabText,
  windowTitle,
  type Tab,
} from "../editor/documents";
import { wrapCompartment, wrapExtension, fontCompartment, fontExtension } from "../editor/setup";
import type { Encoding, Eol } from "../types";
import * as api from "../api";

export function useDocuments() {
  const tabs = ref<Tab[]>([]);
  const activeId = ref<number | null>(null);
  const view = shallowRef<EditorView | null>(null);

  /**
   * 内容版本号。
   *
   * 凡是通过 `view.setState()` 替换编辑器内容的操作（切换标签、重新加载），
   * 都不会经过 dispatch，因而不会触发 docChanged。外部（预览）若只依赖
   * dispatch 回调就会漏更新。这里让每条会改变内容的路径都递增它。
   */
  const contentVersion = ref(0);

  function bumpContentVersion() {
    contentVersion.value++;
  }

  /** 全局的自动换行与字号，对所有标签生效（记事本的设置也是全局的）。 */
  const wordWrap = ref(false);
  const zoom = ref(100);
  const statusBarVisible = ref(true);
  const fontFamily = ref("Consolas");
  const fontSizePt = ref(11);
  const fontBold = ref(false);
  const fontItalic = ref(false);

  const activeTab = computed(() => tabs.value.find((t) => t.id === activeId.value));

  /** 字号 CSS 值。记事本缩放的是文字大小，不是窗口。 */
  const fontSizeCss = computed(() => {
    const pt = (fontSizePt.value * zoom.value) / 100;
    return `${pt.toFixed(2)}pt`;
  });

  const anyDirty = computed(() => tabs.value.some((t) => t.dirty));

  function applyZoom() {
    const v = view.value;
    if (v) {
      v.dispatch({
        effects: fontCompartment.reconfigure(
          fontExtension(fontSizeCss.value, fontBold.value, fontItalic.value),
        ),
      });
    }
  }

  function applyWordWrap() {
    const v = view.value;
    if (v) {
      v.dispatch({
        effects: wrapCompartment.reconfigure(wrapExtension(wordWrap.value)),
      });
    }
  }

  function attachView(instance: EditorView) {
    view.value = instance;
  }

  /** 把当前 view 的状态存回它所属的标签。 */
  function persistActiveState() {
    const v = view.value;
    const tab = activeTab.value;
    if (v && tab) setState(tab.id, v.state);
  }

  /** 切到指定标签。 */
  function activate(id: number) {
    if (id === activeId.value) return;
    persistActiveState();
    const tab = tabs.value.find((t) => t.id === id);
    if (!tab) return;
    activeId.value = id;
    const v = view.value;
    const state = getState(id);
    if (v && state) v.setState(state);
    // setState 不经过 dispatch，需显式通知内容已变
    bumpContentVersion();
    syncTitle();
  }

  function syncTitle() {
    void api.setWindowTitle(windowTitle(activeTab.value));
  }

  /** 把指定标签的状态装进编辑器。 */
  function loadIntoView(id: number, focus = true) {
    const v = view.value;
    const state = getState(id);
    if (v && state) {
      v.setState(state);
      if (focus) v.focus();
      // setState 不经过 dispatch，统一在这里通知内容已变，
      // 覆盖新建标签、打开文件、外部改动重载三条路径
      bumpContentVersion();
    }
  }

  /** 新建空标签。 */
  function newTab(): Tab {
    persistActiveState();
    const tab = createTab(wordWrap.value, fontSizeCss.value);
    tabs.value.push(tab);
    activeId.value = tab.id;
    loadIntoView(tab.id);
    syncTitle();
    return tab;
  }

  /** 从磁盘路径打开文件，作为新标签。 */
  async function openPath(path: string): Promise<void> {
    const existing = tabs.value.find((t) => t.path === path);
    if (existing) {
      activate(existing.id);
      return;
    }
    const info = await api.readDocument(path);
    persistActiveState();
    const tab = createTab(wordWrap.value, fontSizeCss.value, info.text, {
      path: info.path,
      encoding: info.encoding,
      eol: info.eol,
      modifiedMs: info.modifiedMs,
    });
    tabs.value.push(tab);
    activeId.value = tab.id;
    loadIntoView(tab.id);
    syncTitle();
  }

  /** 关闭标签。若关的是最后一个，会自动补一个空标签。 */
  function closeTab(id: number) {
    const idx = tabs.value.findIndex((t) => t.id === id);
    if (idx < 0) return;
    const wasActive = tabs.value[idx]!.id === activeId.value;
    tabs.value.splice(idx, 1);
    dropState(id);

    if (tabs.value.length === 0) {
      activeId.value = null;
      newTab();
      return;
    }

    if (wasActive) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)]!;
      activeId.value = null;
      activate(next.id);
    }
  }

  /** 把当前标签的内容写到指定路径。 */
  async function saveActiveAs(
    path: string,
    encoding: Encoding,
    eol: Eol,
    force = false,
  ): Promise<api.WriteResult> {
    const tab = activeTab.value;
    if (!tab) throw new Error("没有活动标签");

    persistActiveState();

    const result = await api.writeDocument(
      { path, text: tabText(tab), encoding, eol },
      force,
    );

    if (result.ok) {
      tab.path = path;
      tab.name = api.baseName(path);
      tab.encoding = encoding;
      tab.eol = eol;
      tab.dirty = false;
      tab.modifiedMs = result.modifiedMs;
      tab.reloadPending = false;
      syncTitle();
    }
    return result;
  }

  /** 保存到已有路径。 */
  async function saveActive(force = false): Promise<api.WriteResult | null> {
    const tab = activeTab.value;
    if (!tab?.path) return null;
    return saveActiveAs(tab.path, tab.encoding, tab.eol, force);
  }

  /** 标记当前标签为脏。 */
  function markDirty() {
    const tab = activeTab.value;
    if (tab && !tab.dirty) {
      tab.dirty = true;
      syncTitle();
    }
  }

  /** 用磁盘内容覆盖当前标签（外部改动后重新加载）。 */
  async function reloadActive(): Promise<void> {
    const tab = activeTab.value;
    if (!tab?.path) return;
    const info = await api.readDocument(tab.path);
    const state = getState(tab.id);
    if (state) {
      const next = state.update({
        changes: { from: 0, to: state.doc.length, insert: info.text },
      }).state;
      setState(tab.id, next);
    }
    tab.encoding = info.encoding;
    tab.eol = info.eol;
    tab.dirty = false;
    tab.modifiedMs = info.modifiedMs;
    tab.reloadPending = false;
    loadIntoView(tab.id, false);
    syncTitle();
  }

  /** 切换当前标签的换行符。这会立即把标签标记为脏。 */
  function setActiveEol(eol: Eol) {
    const tab = activeTab.value;
    if (!tab || tab.eol === eol) return;
    tab.eol = eol;
    tab.dirty = true;
    syncTitle();
  }

  return {
    tabs,
    activeId,
    activeTab,
    view,
    wordWrap,
    zoom,
    statusBarVisible,
    fontFamily,
    fontSizePt,
    fontBold,
    fontItalic,
    fontSizeCss,
    anyDirty,
    contentVersion,
    attachView,
    persistActiveState,
    activate,
    newTab,
    openPath,
    closeTab,
    saveActive,
    saveActiveAs,
    markDirty,
    reloadActive,
    setActiveEol,
    applyZoom,
    applyWordWrap,
    syncTitle,
  };
}
