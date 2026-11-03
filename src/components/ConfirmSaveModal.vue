<script setup lang="ts">
/**
 * 未保存改动提示。
 *
 * 三个选项（保存 / 不保存 / 取消）与记事本一致，但视觉是独立设计：
 * 圆角卡片、品牌色主按钮、半透明遮罩，而非 Win32 的方框对话框。
 */
defineProps<{
  visible: boolean;
  /** 文件名，用于提示文案 */
  name: string;
}>();

const emit = defineEmits<{
  (e: "save"): void;
  (e: "discard"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <Transition name="pm-fade">
    <div v-if="visible" class="overlay" @keydown.esc="emit('cancel')">
      <div class="dialog" role="dialog" aria-modal="true">
        <div class="icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="22" height="22">
            <path
              d="M12 3 L22 20 H2 Z"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linejoin="round"
            />
            <path d="M12 10 V14" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
            <circle cx="12" cy="17" r="1.1" fill="currentColor" />
          </svg>
        </div>

        <div class="content">
          <p class="title">是否保存更改？</p>
          <p class="message">
            <span class="filename">{{ name }}</span> 有未保存的修改。
          </p>
        </div>

        <div class="buttons">
          <button type="button" class="btn btn-ghost" @click="emit('cancel')">取消</button>
          <button type="button" class="btn btn-ghost" @click="emit('discard')">不保存</button>
          <button type="button" class="btn btn-primary" @click="emit('save')">保存</button>
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
  z-index: 100;
}

.dialog {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0 14px;
  width: 420px;
  max-width: calc(100vw - 48px);
  padding: 20px 20px 16px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-lg);
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: var(--pm-radius-md);
  background: var(--pm-brand-soft);
  color: var(--pm-brand-strong);
}

.title {
  margin: 2px 0 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--pm-fg);
}

.message {
  margin: 0;
  font-size: 12px;
  line-height: 1.6;
  color: var(--pm-fg-muted);
}

.filename {
  color: var(--pm-fg);
  font-weight: 500;
  word-break: break-all;
}

.buttons {
  grid-column: 1 / -1;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn {
  min-width: 76px;
  height: 30px;
  padding: 0 14px;
  border-radius: var(--pm-radius-md);
  font-size: 12px;
  transition:
    background var(--pm-transition),
    border-color var(--pm-transition),
    color var(--pm-transition);
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

/* 遮罩淡入、卡片轻微上浮 */
.pm-fade-enter-active,
.pm-fade-leave-active {
  transition: opacity 140ms ease;
}

.pm-fade-enter-active .dialog,
.pm-fade-leave-active .dialog {
  transition: transform 140ms ease;
}

.pm-fade-enter-from,
.pm-fade-leave-to {
  opacity: 0;
}

.pm-fade-enter-from .dialog,
.pm-fade-leave-to .dialog {
  transform: translateY(-6px) scale(0.98);
}
</style>
