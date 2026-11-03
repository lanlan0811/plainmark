<script setup lang="ts">
/**
 * 更新提示对话框。
 *
 * 三种形态共用一个组件：发现新版本（询问）、下载中（进度）、出错。
 * 安装完成后应用会直接重启，因此不需要「已就绪」这一态。
 */
import { computed } from "vue";
import type { UpdatePhase } from "../composables/useUpdater";

const props = defineProps<{
  phase: UpdatePhase;
  version: string;
  /** 当前运行版本，用于在提示里对比 */
  currentVersion: string;
  notes: string;
  progress: number | null;
  errorMessage: string;
}>();

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "dismiss"): void;
}>();

/** 只有这三种状态需要显示对话框。 */
const visible = computed(
  () =>
    props.phase === "available" ||
    props.phase === "downloading" ||
    props.phase === "ready" ||
    props.phase === "error",
);

/** 更新说明可能很长，截断显示避免对话框过高。 */
const shortNotes = computed(() => {
  const text = props.notes.trim();
  if (!text) return "";
  const lines = text.split("\n").slice(0, 8).join("\n");
  return lines.length > 400 ? `${lines.slice(0, 400)}…` : lines;
});

const progressText = computed(() =>
  props.progress === null ? "正在下载…" : `正在下载 ${Math.round(props.progress)}%`,
);
</script>

<template>
  <Transition name="pm-fade">
    <div v-if="visible" class="overlay">
      <div class="dialog" role="dialog" aria-modal="true">
        <!-- 发现新版本 -->
        <template v-if="phase === 'available'">
          <div class="head">
            <div class="icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path
                  d="M12 4 V15 M12 15 L7.5 10.5 M12 15 L16.5 10.5"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.9"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
                <path
                  d="M4 18 H20"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.9"
                  stroke-linecap="round"
                />
              </svg>
            </div>
            <div>
              <p class="title">发现新版本 {{ version }}</p>
              <p class="subtitle">当前版本 {{ currentVersion }}</p>
            </div>
          </div>

          <pre v-if="shortNotes" class="notes">{{ shortNotes }}</pre>

          <div class="buttons">
            <button type="button" class="btn btn-ghost" @click="emit('dismiss')">
              稍后再说
            </button>
            <button type="button" class="btn btn-primary" @click="emit('confirm')">
              立即更新
            </button>
          </div>
        </template>

        <!-- 下载中 / 安装中 -->
        <template v-else-if="phase === 'downloading' || phase === 'ready'">
          <div class="head">
            <div class="icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path
                  d="M12 4 V15 M12 15 L7.5 10.5 M12 15 L16.5 10.5"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.9"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
                <path
                  d="M4 18 H20"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.9"
                  stroke-linecap="round"
                />
              </svg>
            </div>
            <div>
              <p class="title">
                {{ phase === "ready" ? "正在安装…" : progressText }}
              </p>
              <p class="subtitle">更新到 {{ version }}，完成后应用会自动重启</p>
            </div>
          </div>

          <div class="bar" :class="{ indeterminate: progress === null }">
            <div class="bar-fill" :style="{ width: `${progress ?? 100}%` }"></div>
          </div>
        </template>

        <!-- 出错 -->
        <template v-else>
          <div class="head">
            <div class="icon icon-error" aria-hidden="true">
              <svg viewBox="0 0 24 24" width="20" height="20">
                <circle
                  cx="12"
                  cy="12"
                  r="9"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                />
                <path
                  d="M12 7.5 V13"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linecap="round"
                />
                <circle cx="12" cy="16.3" r="1.1" fill="currentColor" />
              </svg>
            </div>
            <div>
              <p class="title">更新失败</p>
              <p class="subtitle">{{ errorMessage || "请稍后重试" }}</p>
            </div>
          </div>

          <div class="buttons">
            <button type="button" class="btn btn-ghost" @click="emit('dismiss')">
              关闭
            </button>
          </div>
        </template>
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
  z-index: 110;
}

.dialog {
  width: 460px;
  max-width: calc(100vw - 48px);
  padding: 20px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-lg);
}

.head {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 38px;
  height: 38px;
  border-radius: var(--pm-radius-md);
  background: var(--pm-brand-soft);
  color: var(--pm-brand-strong);
}

.icon-error {
  background: #fdecec;
  color: #c0392b;
}

.title {
  margin: 2px 0 4px;
  font-size: 13px;
  font-weight: 600;
  color: var(--pm-fg);
}

.subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--pm-fg-muted);
  line-height: 1.5;
  word-break: break-word;
}

.notes {
  max-height: 180px;
  margin: 16px 0 0;
  padding: 10px 12px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-chrome);
  color: var(--pm-fg-muted);
  font-family: inherit;
  font-size: 11px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  overflow: auto;
}

.bar {
  height: 6px;
  margin-top: 18px;
  border-radius: 999px;
  background: var(--pm-chrome-strong);
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 999px;
  background: var(--pm-brand);
  transition: width 160ms ease;
}

/* 总大小未知时用流动条纹表示进行中 */
.bar.indeterminate .bar-fill {
  width: 40% !important;
  animation: pm-slide 1.1s ease-in-out infinite;
}

@keyframes pm-slide {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(250%);
  }
}

.buttons {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.btn {
  min-width: 84px;
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
