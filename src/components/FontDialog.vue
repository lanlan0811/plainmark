<script setup lang="ts">
/**
 * 字体对话框。
 *
 * WebView2 没有原生字体选择器，所以自建一个简化版：字体族 + 字号。
 * 记事本的字体对话框还有字形（粗体/斜体），这里一并提供。
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
  <div v-if="visible" class="overlay">
    <div class="dialog" role="dialog" aria-modal="true">
      <p class="title">字体</p>

      <div class="row">
        <label for="font-family">字体(F):</label>
        <select id="font-family" v-model="localFamily">
          <option v-for="f in FAMILIES" :key="f" :value="f">{{ f }}</option>
        </select>
      </div>

      <div class="row">
        <label for="font-style">字形(Y):</label>
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
        <label for="font-size">大小(S):</label>
        <input id="font-size" v-model.number="localSize" type="number" min="6" max="72" step="1" />
      </div>

      <div class="preview" :style="{
        fontFamily: localFamily,
        fontSize: `${localSize}pt`,
        fontWeight: localBold ? 'bold' : 'normal',
        fontStyle: localItalic ? 'italic' : 'normal',
      }">
        AaBbYyZz 示例文本
      </div>

      <div class="buttons">
        <button type="button" class="btn" @click="apply">确定</button>
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
  width: 400px;
  padding: 16px 18px 12px;
  background: #f0f0f0;
  border: 1px solid #b0b0b0;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  font-size: 12px;
}

.title {
  margin: 0 0 14px;
  font-weight: 600;
}

.row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.row label {
  width: 70px;
  flex: none;
}

.row select,
.row input {
  flex: 1;
  height: 24px;
  padding: 0 6px;
  border: 1px solid #adadad;
  background: #fff;
  font-family: inherit;
  font-size: 12px;
}

.preview {
  height: 68px;
  margin: 14px 0;
  padding: 8px;
  border: 1px solid #adadad;
  background: #fff;
  overflow: hidden;
  line-height: 1.4;
}

.buttons {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 6px;
}

.btn {
  min-width: 76px;
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
</style>
