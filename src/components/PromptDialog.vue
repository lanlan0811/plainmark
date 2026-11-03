<script setup lang="ts">
/**
 * 通用文本输入对话框。
 *
 * 为什么不直接用 `window.prompt()`：在 WebView 里它是浏览器原生实现，
 * 弹窗标题会显示网页的 origin（`tauri.localhost 显示`），与桌面应用的观感
 * 严重不符，也无法定制样式与校验。这里自建一个。
 */
import { ref, watch, nextTick } from "vue";

const props = defineProps<{
  visible: boolean;
  title: string;
  /** 输入框前的说明文字 */
  label?: string;
  /** 初始值 */
  initial?: string;
  placeholder?: string;
  /** 只接受正整数时启用数字校验 */
  numeric?: boolean;
}>();

const emit = defineEmits<{
  (e: "confirm", value: string): void;
  (e: "cancel"): void;
}>();

const value = ref("");
const inputEl = ref<HTMLInputElement | null>(null);
/** 校验失败时的提示 */
const error = ref("");

watch(
  () => props.visible,
  async (open) => {
    if (!open) return;
    value.value = props.initial ?? "";
    error.value = "";
    await nextTick();
    inputEl.value?.focus();
    inputEl.value?.select();
  },
);

function confirm() {
  const text = value.value.trim();

  if (props.numeric) {
    const n = Number.parseInt(text, 10);
    if (!Number.isFinite(n) || n < 1) {
      error.value = "请输入大于 0 的整数";
      return;
    }
  }

  emit("confirm", text);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    confirm();
  } else if (e.key === "Escape") {
    e.preventDefault();
    emit("cancel");
  }
}
</script>

<template>
  <Transition name="pm-fade">
    <div v-if="visible" class="overlay" @keydown.esc="emit('cancel')">
      <div class="dialog" role="dialog" aria-modal="true">
        <p class="title">{{ title }}</p>

        <label v-if="label" class="label" for="pm-prompt-input">{{ label }}</label>
        <input
          id="pm-prompt-input"
          ref="inputEl"
          v-model="value"
          class="input"
          :class="{ invalid: !!error }"
          type="text"
          :placeholder="placeholder"
          @keydown="onKeydown"
        />

        <p v-if="error" class="error">{{ error }}</p>

        <div class="buttons">
          <button type="button" class="btn btn-ghost" @click="emit('cancel')">取消</button>
          <button type="button" class="btn btn-primary" @click="confirm">确定</button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(31, 32, 51, 0.28);
  backdrop-filter: blur(2px);
  z-index: 120;
}

.dialog {
  width: 340px;
  max-width: calc(100vw - 48px);
  padding: 20px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-lg);
}

.title {
  margin: 0 0 14px;
  font-size: 13px;
  font-weight: 600;
  color: var(--pm-fg);
}

.label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  color: var(--pm-fg-muted);
}

.input {
  width: 100%;
  height: 32px;
  padding: 0 10px;
  border: 1px solid var(--pm-border-strong);
  border-radius: var(--pm-radius-md);
  background: var(--pm-bg);
  color: var(--pm-fg);
  font-family: inherit;
  font-size: 13px;
  transition: border-color var(--pm-transition);
}

.input:focus {
  border-color: var(--pm-brand);
  outline: none;
}

.input.invalid {
  border-color: #d9534f;
}

.error {
  margin: 6px 0 0;
  font-size: 11px;
  color: #c0392b;
}

.buttons {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}

.btn {
  min-width: 76px;
  height: 30px;
  padding: 0 14px;
  border-radius: var(--pm-radius-md);
  font-size: 12px;
  transition:
    background var(--pm-transition),
    border-color var(--pm-transition);
}

.btn-ghost {
  border: 1px solid var(--pm-border-strong);
  background: var(--pm-bg);
  color: var(--pm-fg);
}

.btn-ghost:hover {
  background: var(--pm-chrome);
  border-color: var(--pm-fg-subtle);
}

.btn-primary {
  border: 1px solid var(--pm-brand-strong);
  background: var(--pm-brand);
  color: #ffffff;
  font-weight: 500;
}

.btn-primary:hover {
  background: var(--pm-brand-strong);
}

.pm-fade-enter-active,
.pm-fade-leave-active {
  transition: opacity 140ms ease;
}

.pm-fade-enter-from,
.pm-fade-leave-to {
  opacity: 0;
}
</style>
