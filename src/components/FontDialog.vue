<script setup lang="ts">
/**
 * 字体对话框。
 *
 * WebView2 没有原生字体选择器，所以自建。功能与记事本的字体设置等价
 * （字体族 / 字形 / 字号 + 预览），视觉沿用本应用的圆角卡片风格。
 */
import { ref, watch } from "vue";

const props = defineProps<{
  visible: boolean;
  family: string;
  sizePt: number;
  bold: boolean;
  italic: boolean;
}>();

const emit = defineEmits<{
  (e: "apply", value: { family: string; sizePt: number; bold: boolean; italic: boolean }): void;
  (e: "cancel"): void;
}>();

/** 常见等宽与正文候选，不枚举系统全部字体以保持简单。 */
const FAMILIES = [
  "Consolas",
  "Courier New",
  "Lucida Console",
  "Cascadia Mono",
  "Microsoft YaHei UI",
  "SimSun",
  "Segoe UI",
];

const localFamily = ref(props.family);
const localSize = ref(props.sizePt);
const localBold = ref(props.bold);
const localItalic = ref(props.italic);

// 每次打开时把外部值同步进来
watch(
  () => props.visible,
  (open) => {
    if (open) {
      localFamily.value = props.family;
      localSize.value = props.sizePt;
      localBold.value = props.bold;
      localItalic.value = props.italic;
    }
  },
);

function apply() {
  const size = Math.min(72, Math.max(6, Math.round(localSize.value)));
  emit("apply", {
    family: localFamily.value,
    sizePt: size,
    bold: localBold.value,
    italic: localItalic.value,
  });
}
</script>

<template>
  <Transition name="pm-fade">
    <div v-if="visible" class="overlay">
      <div class="dialog" role="dialog" aria-modal="true">
        <p class="title">字体</p>

        <div class="row">
          <label for="font-family">字体</label>
          <select id="font-family" v-model="localFamily">
            <option v-for="f in FAMILIES" :key="f" :value="f">{{ f }}</option>
          </select>
        </div>

        <div class="row">
          <label for="font-style">字形</label>
          <select
            id="font-style"
            :value="localBold ? (localItalic ? 'bi' : 'b') : localItalic ? 'i' : 'r'"
            @change="
              (e) => {
                const v = (e.target as HTMLSelectElement).value;
                localBold = v === 'b' || v === 'bi';
                localItalic = v === 'i' || v === 'bi';
              }
            "
          >
            <option value="r">常规</option>
            <option value="i">斜体</option>
            <option value="b">粗体</option>
            <option value="bi">粗斜体</option>
          </select>
        </div>

        <div class="row">
          <label for="font-size">字号</label>
          <input id="font-size" v-model.number="localSize" type="number" min="6" max="72" step="1" />
        </div>

        <div
          class="preview"
          :style="{
            fontFamily: localFamily,
            fontSize: `${localSize}pt`,
            fontWeight: localBold ? 'bold' : 'normal',
            fontStyle: localItalic ? 'italic' : 'normal',
          }"
        >
          AaBbYyZz 示例文本
        </div>

        <div class="buttons">
          <button type="button" class="btn btn-ghost" @click="emit('cancel')">取消</button>
          <button type="button" class="btn btn-primary" @click="apply">确定</button>
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
  width: 420px;
  max-width: calc(100vw - 48px);
  padding: 20px 20px 16px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-lg);
}

.title {
  margin: 0 0 16px;
  font-size: 13px;
  font-weight: 600;
  color: var(--pm-fg);
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}

.row label {
  flex: none;
  width: 48px;
  color: var(--pm-fg-muted);
  font-size: 12px;
}

.row select,
.row input {
  flex: 1;
  height: 30px;
  padding: 0 8px;
  border: 1px solid var(--pm-border-strong);
  border-radius: var(--pm-radius-md);
  background: var(--pm-bg);
  color: var(--pm-fg);
  font-family: inherit;
  font-size: 12px;
  transition: border-color var(--pm-transition);
}

.row select:focus,
.row input:focus {
  border-color: var(--pm-brand);
  outline: none;
}

.preview {
  height: 76px;
  margin: 16px 0 4px;
  padding: 10px 12px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-chrome);
  overflow: hidden;
  line-height: 1.5;
  color: var(--pm-fg);
}

.buttons {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
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
