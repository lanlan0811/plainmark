/**
 * 自动更新。
 *
 * 流程：启动后静默检查 → 有新版本则弹窗询问 → 用户同意后下载（带进度）
 * → 安装 → 重启应用。
 *
 * 关于安装包类型的区分：**不需要我们判断**。Tauri 的更新器在 Windows 上会
 * 读取当前应用的安装来源（MSI 还是 NSIS），然后在更新清单里按
 * `windows-x86_64-msi` / `windows-x86_64-nsis` 的键去找对应产物，
 * 找不到才回退到通用的 `windows-x86_64`。因此只要 Release 里的 latest.json
 * 同时提供这两个键，用 exe 装的用户会拿到 exe、用 msi 装的会拿到 msi。
 */

import { ref, readonly } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

/** 更新状态机。 */
export type UpdatePhase =
  | "idle" // 未检查
  | "checking" // 检查中
  | "available" // 发现新版本，等待用户决定
  | "downloading" // 下载中
  | "ready" // 下载完成，等待安装
  | "error"; // 出错

const phase = ref<UpdatePhase>("idle");
/** 新版本号 */
const version = ref("");
/** 更新说明（来自 Release 正文） */
const notes = ref("");
/** 下载进度 0–100；总大小未知时为 null */
const progress = ref<number | null>(null);
/** 错误信息 */
const errorMessage = ref("");

/** 当前的更新句柄，供下载/安装使用。 */
let pending: Update | null = null;

/** 检查是否有新版本。静默失败——网络不通时不应打扰用户。 */
export async function checkForUpdate(): Promise<boolean> {
  phase.value = "checking";
  try {
    const update = await check();
    if (!update) {
      phase.value = "idle";
      return false;
    }

    pending = update;
    version.value = update.version;
    notes.value = update.body ?? "";
    phase.value = "available";
    return true;
  } catch (e) {
    // 离线、GitHub 不可达、清单缺失等都会走到这里。
    // 自动检查是后台行为，失败时保持沉默，不弹错误。
    phase.value = "idle";
    errorMessage.value = e instanceof Error ? e.message : String(e);
    return false;
  }
}

/** 下载并安装更新，完成后重启应用。 */
export async function downloadAndInstall(): Promise<void> {
  if (!pending) return;

  phase.value = "downloading";
  progress.value = 0;

  try {
    let downloaded = 0;
    let total: number | null = null;

    await pending.download((event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength ?? null;
          progress.value = total ? 0 : null;
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          progress.value = total ? Math.min(100, (downloaded / total) * 100) : null;
          break;
        case "Finished":
          progress.value = 100;
          break;
      }
    });

    phase.value = "ready";
    await pending.install();

    // 安装完成后重启，让新版本生效
    await relaunch();
  } catch (e) {
    phase.value = "error";
    errorMessage.value = e instanceof Error ? e.message : String(e);
  }
}

/** 用户选择稍后再说。 */
export function dismissUpdate(): void {
  phase.value = "idle";
  pending = null;
}

export function useUpdater() {
  return {
    phase: readonly(phase),
    version: readonly(version),
    notes: readonly(notes),
    progress: readonly(progress),
    errorMessage: readonly(errorMessage),
  };
}
