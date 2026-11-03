/**
 * 标签页与文档状态。
 *
 * 设计要点：`Tab` 只保存**可序列化的元信息**，编辑器的 `EditorState` 单独放在
 * 一个非响应式的 Map 里。原因有二：
 *   1. `EditorState` 含私有字段，一旦被 Vue 的 `ref()` 深度解包就会丢失类型
 *      （UnwrapRef 会把它拆成结构化对象，private 成员消失）；
 *   2. 它是不可变的大对象，不需要也不应该被深度追踪。
 *
 * 每个标签各持一份 EditorState，因此撤销历史天然按标签隔离。
 */

import type { EditorState } from "@codemirror/state";
import type { Encoding, Eol } from "../types";
import { UNTITLED, baseName } from "../api";
import { createEditorState } from "./setup";

/** 标签的元信息，这部分是响应式的。 */
export interface Tab {
  id: number;
  /** 磁盘路径，null 表示尚未保存的新文档 */
  path: string | null;
  /** 显示名：文件名或「无标题」 */
  name: string;
  encoding: Encoding;
  eol: Eol;
  /** 自打开/保存以来是否有改动 */
  dirty: boolean;
  /** 磁盘上的最后修改时间，用于外部改动检测 */
  modifiedMs: number | null;
  /** 外部改动提示是否已被用户处理过，避免反复弹窗 */
  reloadPending: boolean;
}

let nextId = 1;

/** 编辑器状态存储。非响应式——见文件头说明。 */
const states = new Map<number, EditorState>();

export function getState(id: number): EditorState | undefined {
  return states.get(id);
}

export function setState(id: number, state: EditorState): void {
  states.set(id, state);
}

export function dropState(id: number): void {
  states.delete(id);
}

/** 新建一个空标签，并初始化它的编辑器状态。 */
export function createTab(
  wordWrap: boolean,
  fontSize: string,
  text = "",
  init?: Partial<Pick<Tab, "path" | "encoding" | "eol" | "modifiedMs">>,
): Tab {
  const path = init?.path ?? null;
  const id = nextId++;
  states.set(id, createEditorState(text, wordWrap, fontSize));
  return {
    id,
    path,
    name: path ? baseName(path) : UNTITLED,
    encoding: init?.encoding ?? "Utf8",
    eol: init?.eol ?? "Crlf",
    dirty: false,
    modifiedMs: init?.modifiedMs ?? null,
    reloadPending: false,
  };
}

/** 取出标签当前的文本（换行符为 \n）。 */
export function tabText(tab: Tab): string {
  return states.get(tab.id)?.doc.toString() ?? "";
}

/** 生成标题栏文字：脏时前缀 *，格式对齐记事本 `*名字 - plainmark`。 */
export function windowTitle(tab: Tab | undefined): string {
  if (!tab) return "plainmark";
  return `${tab.dirty ? "*" : ""}${tab.name} - plainmark`;
}

/** 标签上的显示文字：脏时后缀 *。 */
export function tabLabel(tab: Tab): string {
  return tab.dirty ? `${tab.name} *` : tab.name;
}
