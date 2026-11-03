/**
 * CodeMirror 6 配置。
 *
 * 设计目标：一个朴素的 Markdown 写作面，**不**像代码编辑器。
 * 因此刻意不引入行号栏、折叠、括号匹配、自动补全、活动行高亮等扩展，
 * 并关闭自动缩进与自动换行（写作时默认不折行，与纯文本编辑器一致）。
 *
 * 唯一的额外能力是 Markdown 语法高亮——只改字符颜色，源码始终完整可见可编辑，
 * 不提供预览面板。
 *
 * 关于外观：功能上对齐 Windows 记事本，但视觉是独立设计（见 style.css 的说明），
 * 刻意不复刻记事本界面，以避免 trade dress 风险。
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

/**
 * 编辑器外观。
 *
 * 文字区域本身保持简洁（白底、等宽字体、无行号），这是 Markdown 写作该有的样子；
 * 但周边的面板、输入框、按钮用品牌语言重做，与记事本的纯灰直角色调区分开。
 */
function editorTheme(fontSize: string, bold = false, italic = false): Extension {
  return EditorView.theme({
    "&": {
      height: "100%",
      fontSize,
      backgroundColor: "var(--pm-bg)",
      color: "var(--pm-fg)",
    },
    ".cm-scroller": {
      fontFamily: "var(--pm-font-family)",
      lineHeight: "1.6",
      overflow: "auto",
      // 字形设置来自「格式 → 字体」
      fontWeight: bold ? "bold" : "normal",
      fontStyle: italic ? "italic" : "normal",
    },
    ".cm-content": {
      // 比记事本的 2px 留白更舒展，长时间写作更舒适
      padding: "12px 16px",
      caretColor: "var(--pm-caret)",
    },
    ".cm-line": {
      padding: "0",
    },
    "&.cm-focused": {
      outline: "none",
    },
    ".cm-selectionBackground, ::selection": {
      backgroundColor: "var(--pm-selection) !important",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: "var(--pm-caret)",
      borderLeftWidth: "2px",
    },

    /* ---- 搜索面板：圆角卡片，与品牌一致 ---- */
    ".cm-panels": {
      backgroundColor: "var(--pm-chrome)",
      color: "var(--pm-fg)",
      fontFamily: '"Segoe UI", "Microsoft YaHei UI", system-ui, sans-serif',
      fontSize: "12px",
      borderBottom: "1px solid var(--pm-border)",
    },
    ".cm-panel.cm-search": {
      padding: "8px 10px",
    },
    ".cm-panel.cm-search input[type=text]": {
      height: "24px",
      padding: "0 8px",
      border: "1px solid var(--pm-border-strong)",
      borderRadius: "var(--pm-radius-sm)",
      background: "var(--pm-bg)",
      color: "var(--pm-fg)",
      fontFamily: "inherit",
      fontSize: "12px",
    },
    ".cm-panel.cm-search input[type=text]:focus": {
      borderColor: "var(--pm-brand)",
      outline: "none",
    },
    ".cm-panel.cm-search button": {
      height: "24px",
      padding: "0 10px",
      border: "1px solid var(--pm-border-strong)",
      borderRadius: "var(--pm-radius-sm)",
      background: "var(--pm-bg)",
      color: "var(--pm-fg)",
      fontFamily: "inherit",
      fontSize: "12px",
      transition: "background 120ms ease, border-color 120ms ease",
    },
    ".cm-panel.cm-search button:hover": {
      background: "var(--pm-brand-soft)",
      borderColor: "var(--pm-brand-border)",
      color: "var(--pm-brand-strong)",
    },
    ".cm-panel.cm-search label": {
      fontSize: "12px",
      color: "var(--pm-fg-muted)",
    },
    // 关闭按钮用圆形悬停，替代 CodeMirror 默认的方框
    ".cm-panel.cm-search [name=close]": {
      position: "absolute",
      top: "6px",
      right: "8px",
      width: "20px",
      height: "20px",
      padding: "0",
      border: "none",
      borderRadius: "var(--pm-radius-sm)",
      background: "transparent",
      color: "var(--pm-fg-muted)",
      fontSize: "16px",
      lineHeight: "18px",
    },
    ".cm-panel.cm-search [name=close]:hover": {
      background: "var(--pm-chrome-strong)",
      color: "var(--pm-fg)",
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

      // 查找/替换面板的中文文案。
      // CodeMirror 内置文案是英文，通过 phrases facet 覆盖。
      // 键名必须与 search 包里 phrase() 的调用参数逐字一致，否则不生效。
      EditorState.phrases.of({
        Find: "查找",
        Replace: "替换",
        next: "下一个",
        previous: "上一个",
        all: "全选匹配",
        "match case": "区分大小写",
        regexp: "正则表达式",
        "by word": "全字匹配",
        replace: "替换",
        "replace all": "全部替换",
        close: "关闭",
        "current match": "当前匹配",
        "on line": "位于第",
        "replaced match on line $": "已替换第 $ 行的匹配项",
        "replaced $ matches": "已替换 $ 处匹配",
        "Go to line": "转到行",
        go: "转到",
      }),

      notepadKeymap,
      insertTab,
      keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),

      fontCompartment.of(editorTheme(fontSize, bold, italic)),
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
  return editorTheme(fontSize, bold, italic);
}

export { EditorView, EditorState };
export type { Extension };
