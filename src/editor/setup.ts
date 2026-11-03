/**
 * CodeMirror 6 配置。
 *
 * 设计目标：让它看起来**不像**代码编辑器，而像记事本。
 * 因此刻意不引入行号栏、折叠、括号匹配、自动补全、活动行高亮等扩展，
 * 并关闭自动缩进与自动换行（记事本默认关闭自动换行）。
 *
 * 唯一比记事本多出来的能力是 Markdown 语法高亮——但只改字符颜色，
 * 源码始终完整可见可编辑，不提供预览面板。
 */

import {
  EditorState,
  type Extension,
  Compartment,
} from "@codemirror/state";
import {
  EditorView,
  keymap,
  drawSelection,
  dropCursor,
  rectangularSelection,
} from "@codemirror/view";
import {
  defaultKeymap,
  history,
  historyKeymap,
  undo,
  redo,
  selectAll,
  insertNewline,
} from "@codemirror/commands";
import {
  syntaxHighlighting,
  HighlightStyle,
  indentUnit,
} from "@codemirror/language";
import { markdown } from "@codemirror/lang-markdown";
import {
  search,
  searchKeymap,
  openSearchPanel,
  findNext,
  findPrevious,
} from "@codemirror/search";
import { tags as t } from "@lezer/highlight";

/** 自动换行开关、缩放字号等运行期可变的配置，用 Compartment 动态替换。 */
export const wrapCompartment = new Compartment();
export const fontCompartment = new Compartment();

/**
 * Markdown 高亮配色。
 * 刻意克制——只用少数几种颜色，避免看起来像 IDE 主题。
 */
const markdownHighlight = HighlightStyle.define([
  // 标题：加粗
  { tag: t.heading, fontWeight: "bold" },
  { tag: t.heading1, fontWeight: "bold" },
  { tag: t.heading2, fontWeight: "bold" },
  { tag: t.heading3, fontWeight: "bold" },
  // 强调
  { tag: t.strong, fontWeight: "bold" },
  { tag: t.emphasis, fontStyle: "italic" },
  { tag: t.strikethrough, textDecoration: "line-through" },
  // 行内代码与代码块：等宽 + 淡色底纹
  { tag: t.monospace, fontFamily: "var(--pm-font-family)", color: "#a31515" },
  // 链接
  { tag: t.link, color: "#0451a5", textDecoration: "underline" },
  { tag: t.url, color: "#0451a5" },
  // 引用
  { tag: t.quote, color: "#4a4a4a", fontStyle: "italic" },
  // 列表标记
  { tag: t.list, color: "#0451a5" },
  // 标记符号（# * ` > 等）弱化，让正文更突出
  { tag: t.processingInstruction, color: "#9a9a9a" },
  { tag: t.contentSeparator, color: "#9a9a9a" },
]);

/** 编辑器外观：白底黑字，等宽字体，无内边距修饰。 */
function notepadTheme(fontSize: string, bold = false, italic = false): Extension {
  return EditorView.theme({
    "&": {
      height: "100%",
      fontSize,
      backgroundColor: "var(--pm-bg)",
      color: "var(--pm-fg)",
    },
    ".cm-scroller": {
      fontFamily: "var(--pm-font-family)",
      lineHeight: "1.2",
      overflow: "auto",
      // 字形设置来自「格式 → 字体」，与记事本一致地作用于整个编辑区
      fontWeight: bold ? "bold" : "normal",
      fontStyle: italic ? "italic" : "normal",
    },
    ".cm-content": {
      // 记事本正文四周有约 2px 的内边距感
      padding: "2px 4px",
      caretColor: "#000000",
    },
    ".cm-line": {
      padding: "0",
    },
    "&.cm-focused": {
      // 去掉 CodeMirror 默认的聚焦描边
      outline: "none",
    },
    // 选中色贴近系统
    ".cm-selectionBackground, ::selection": {
      backgroundColor: "var(--pm-selection) !important",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: "#000000",
      borderLeftWidth: "1px",
    },
    // 搜索面板：压成记事本那种朴素的系统对话框观感
    ".cm-panels": {
      backgroundColor: "var(--pm-statusbar-bg)",
      color: "var(--pm-fg)",
      fontFamily: '"Segoe UI", "Microsoft YaHei UI", system-ui, sans-serif',
      fontSize: "12px",
    },
    ".cm-panel.cm-search input, .cm-panel.cm-search button": {
      fontFamily: "inherit",
      fontSize: "12px",
    },
    ".cm-panel.cm-search label": {
      fontSize: "12px",
    },
  });
}

/** Tab 插入字面制表符，而非空格——与记事本一致。 */
const insertTab: Extension = keymap.of([
  {
    key: "Tab",
    run: (view) => {
      view.dispatch(view.state.replaceSelection("\t"));
      return true;
    },
  },
]);

/**
 * 按键映射。
 * 刻意覆盖掉 CodeMirror 默认的 Enter（会自动缩进），改为纯换行。
 */
export const notepadKeymap: Extension = keymap.of([
  { key: "Enter", run: insertNewline },
  { key: "Mod-z", run: undo, preventDefault: true },
  { key: "Mod-y", run: redo, preventDefault: true },
  { key: "Mod-Shift-z", run: redo, preventDefault: true },
  { key: "Mod-a", run: selectAll, preventDefault: true },
  { key: "Mod-f", run: openSearchPanel, preventDefault: true },
  { key: "F3", run: findNext, preventDefault: true },
  { key: "Shift-F3", run: findPrevious, preventDefault: true },
]);

/** 构建一份完整的编辑器状态。每个标签页各持有一份，撤销历史因此天然隔离。 */
export function createEditorState(
  doc: string,
  wordWrap: boolean,
  fontSize: string,
  bold = false,
  italic = false,
): EditorState {
  return EditorState.create({
    doc,
    extensions: [
      history(),
      drawSelection(),
      dropCursor(),
      rectangularSelection(),
      EditorState.allowMultipleSelections.of(true),

      // 制表位 8，与记事本一致
      EditorState.tabSize.of(8),
      indentUnit.of("\t"),

      markdown(),
      syntaxHighlighting(markdownHighlight),

      search({ top: true }),

      notepadKeymap,
      insertTab,
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),

      fontCompartment.of(notepadTheme(fontSize, bold, italic)),
      wrapCompartment.of(wordWrap ? EditorView.lineWrapping : []),
    ],
  });
}

/** 运行期切换自动换行。 */
export function wrapExtension(enabled: boolean): Extension {
  return enabled ? EditorView.lineWrapping : [];
}

/** 运行期切换字号与字形。 */
export function fontExtension(fontSize: string, bold = false, italic = false): Extension {
  return notepadTheme(fontSize, bold, italic);
}

export { EditorView, EditorState };
export type { Extension };
