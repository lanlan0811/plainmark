/** Rust 后端的调用封装。所有文件系统操作都在 Rust 侧完成。 */

import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { DocumentInfo, Encoding, Eol, SaveOptions, WriteResult } from "./types";

/** 打开一个文件并解析出内容、编码与换行符。 */
export function readDocument(path: string): Promise<DocumentInfo> {
  return invoke<DocumentInfo>("read_document", { path });
}

/**
 * 把内容按指定编码与换行符写回磁盘。
 *
 * `force` 为 false 时，若编码无法表示全部字符则不落盘，返回 `lossy: true`；
 * 用户确认后再次调用并置 `force` 为 true。
 */
export function writeDocument(options: SaveOptions, force = false): Promise<WriteResult> {
  return invoke<WriteResult>("write_document", { options, force });
}

/** 启动时通过命令行传入的文件路径。 */
export function getStartupFiles(): Promise<string[]> {
  return invoke<string[]>("get_startup_files");
}

/** 通知后端当前窗口是否有未保存改动，用于关闭窗口前拦截。 */
export function setWindowDirty(dirty: boolean): Promise<void> {
  return invoke("set_window_dirty", { dirty });
}

/** 设置当前窗口标题。 */
export function setWindowTitle(title: string): Promise<void> {
  return invoke("set_title", { title });
}

/** 关闭当前窗口。 */
export function closeWindow(): Promise<void> {
  return invoke("close_window");
}

/** 退出应用。 */
export function exitApp(): Promise<void> {
  return invoke("exit_app");
}

/** 用系统默认浏览器打开帮助页。 */
export function openHelp(): Promise<void> {
  return invoke("open_help");
}

/** 用系统对话框选择要打开的文件。返回 null 表示用户取消。 */
export async function pickOpenPath(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      { name: "Markdown 文档", extensions: ["md", "markdown"] },
      { name: "文本文档", extensions: ["txt"] },
      { name: "所有文件", extensions: ["*"] },
    ],
  });
  return typeof selected === "string" ? selected : null;
}

/** 用系统对话框选择保存位置。返回 null 表示用户取消。 */
export async function pickSavePath(defaultName: string): Promise<string | null> {
  const selected = await save({
    defaultPath: defaultName,
    filters: [
      { name: "Markdown 文档", extensions: ["md"] },
      { name: "所有文件", extensions: ["*"] },
    ],
  });
  return typeof selected === "string" ? selected : null;
}

/** 从完整路径里取出文件名，兼容 Windows 反斜杠。 */
export function baseName(path: string): string {
  const idx = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return idx >= 0 ? path.slice(idx + 1) : path;
}

/** 未保存新文档的显示名，对齐记事本的「无标题」。 */
export const UNTITLED = "无标题";

export type { DocumentInfo, Encoding, Eol, SaveOptions, WriteResult };
