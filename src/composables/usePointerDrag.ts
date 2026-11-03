/**
 * 基于指针事件的拖拽会话。
 *
 * 为什么不用 HTML5 拖放（dragstart / dragover / drop）：
 * Tauri 在 Windows 上默认 `dragDropEnabled: true`，会接管 WebView2 的拖放处理器，
 * **HTML5 拖放事件根本不会触发**。而该选项又是「把 .md 文件拖进窗口打开」所必需的，
 * 无法关闭。因此应用内的拖拽一律走 pointer 事件，与平台拖放互不干扰。
 *
 * 同一时刻只允许一个拖拽会话，所以状态放在模块级而非组件内。
 */

import { ref, readonly } from "vue";

/** 拖拽载荷。 */
export type DragPayload =
  | { kind: "element"; markdown: string; label: string }
  | { kind: "block"; index: number };

const active = ref(false);
const payload = ref<DragPayload | null>(null);
/** 指针当前位置（视口坐标），用于绘制跟随光标的提示 */
const pointer = ref<{ x: number; y: number }>({ x: 0, y: 0 });

/** 指针是否落在某个元素内。 */
export function isPointInside(el: HTMLElement | null, x: number, y: number): boolean {
  if (!el) return false;
  const r = el.getBoundingClientRect();
  return x >= r.left && x <= r.right && y >= r.top && y <= r.bottom;
}

/**
 * 开始一次拖拽会话。
 *
 * 会自动挂上 window 级的 pointermove / pointerup 监听，会话结束时清理。
 */
export function startDrag(
  initial: DragPayload,
  handlers: {
    /** 指针移动。返回 false 可终止会话。 */
    onMove?: (x: number, y: number) => void;
    /** 指针抬起。x/y 为最终位置。 */
    onDrop?: (x: number, y: number) => void;
    /** 会话被取消（Esc 或指针离开窗口） */
    onCancel?: () => void;
  },
): void {
  active.value = true;
  payload.value = initial;

  const move = (e: PointerEvent) => {
    pointer.value = { x: e.clientX, y: e.clientY };
    handlers.onMove?.(e.clientX, e.clientY);
  };

  const up = (e: PointerEvent) => {
    const x = e.clientX;
    const y = e.clientY;
    cleanup();
    handlers.onDrop?.(x, y);
  };

  const key = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      cleanup();
      handlers.onCancel?.();
    }
  };

  function cleanup() {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", up);
    window.removeEventListener("keydown", key);
    active.value = false;
    payload.value = null;
  }

  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", up);
  window.addEventListener("keydown", key);
}

export function usePointerDrag() {
  return {
    active: readonly(active),
    payload: readonly(payload),
    pointer: readonly(pointer),
  };
}
