/** Rust 后端的调用封装。所有文件系统操作都在 Rust 侧完成。 */

import { invoke } from "@tauri-apps/api/core";
import {
  open,
  save,
  message as nativeMessage,
  confirm as nativeConfirm,
} from "@tauri-apps/plugin-dialog";
import type { DocumentInfo, Encoding, Eol, SaveOptions, WriteResult } from "./types";

/**
 * 提示信息。
 *
 * 用 Tauri 的原生对话框而非 `window.alert()`：后者的标题会显示网页 origin
 * （形如「tauri.localhost 显示」），与桌面应用观感严重不符。
 */
export async function alert(message: string, title = "plainmark"): Promise<void> {
  // 用 buttons 而非已废弃的 okLabel
  await nativeMessage(message, { title, kind: "info", buttons: { ok: "确定" } });
}

/** 警告信息（带警告图标）。 */
export async function warn(message: string, title = "plainmark"): Promise<void> {
  await nativeMessage(message, { title, kind: "warning", buttons: { ok: "确定" } });
}

/** 确认对话框。返回用户是否点击了「确定」。 */
export async function confirm(message: string, title = "plainmark"): Promise<boolean> {
  return nativeConfirm(message, {
    title,
    kind: "warning",
    okLabel: "确定",
    cancelLabel: "取消",
  });
}

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

/** 用系统默认浏览器打开开源协议页面。 */
export function openLicense(): Promise<void> {
  return invoke("open_license");
}

/**
 * 上报启动里程碑，用于定位启动耗时。
 *
 * Rust 侧无法感知 WebView 内部的加载进度，只能由前端在关键节点主动上报。
 * 仅在设置 `PLAINMARK_STARTUP_LOG` 环境变量时才会真正输出日志。
 */
export function reportStartupMilestone(name: string): void {
  // 不 await：埋点不应影响启动路径
  void invoke("report_startup_milestone", { name }).catch(() => {});
}

/**
 * 取文件最后修改时间（毫秒时间戳），用于外部改动检测。
 * 返回 null 表示文件已不存在。
 */
export function fileModifiedMs(path: string): Promise<number | null> {
  return invoke<number | null>("file_modified_ms", { path });
}

/** 导入图片的结果。 */
export interface ImportedAsset {
  /** 可写进 Markdown 的相对路径，如 `assets/图片.png` */
  relativePath: string;
  absolutePath: string;
  name: string;
}

/**
 * 把图片复制到文档同目录的 `assets/` 下。
 * 文档尚未保存时没有目录可放素材，需先「另存为」。
 */
export function importImage(docPath: string, sourcePath: string): Promise<ImportedAsset> {
  return invoke<ImportedAsset>("import_image", { docPath, sourcePath });
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
