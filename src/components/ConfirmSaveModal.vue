<script setup lang="ts">
/**
 * 未保存改动提示。三按钮（保存/不保存/取消），对齐记事本。
 *
 * Tauri 的 dialog.ask 只有两个按钮，所以这里自建。
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
  <div v-if="visible" class="overlay" @keydown.esc="emit('cancel')">
    <div class="dialog" role="dialog" aria-modal="true">
      <div class="body">
        <p class="title">plainmark</p>
        <p class="message">是否要将更改保存到 {{ name }}？</p>
      </div>
      <div class="buttons">
        <button type="button" class="btn" @click="emit('save')">保存(S)</button>
        <button type="button" class="btn" @click="emit('discard')">不保存(N)</button>
        <button type="button" class="btn" @click="emit('cancel')">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.12);
  z-index: 100;
}

.dialog {
  min-width: 380px;
  background: #f0f0f0;
  border: 1px solid #b0b0b0;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  font-size: 12px;
}

.body {
  padding: 18px 20px 22px;
}

.title {
  margin: 0 0 12px;
  font-weight: 600;
}

.message {
  margin: 0;
  line-height: 1.5;
}

.buttons {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 14px;
  background: #f0f0f0;
  border-top: 1px solid #dcdcdc;
}

.btn {
  min-width: 80px;
  padding: 4px 12px;
  border: 1px solid #adadad;
  background: #e1e1e1;
  font-family: inherit;
  font-size: 12px;
  cursor: default;
}

.btn:hover {
  background: #e5f1fb;
  border-color: #0078d7;
}

.btn:first-child {
  border-color: #0078d7;
}
</style>
