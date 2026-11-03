<script setup lang="ts">
/**
 * 关于对话框。
 *
 * 原生 message 弹窗只能显示纯文本，信息量太少。这里自建一个，展示
 * 版本、运行环境与项目链接。
 *
 * 环境信息按需加载（打开时才取），避免把 app/os 插件的调用放进启动路径。
 */
import { ref, watch } from "vue";

const props = defineProps<{
  visible: boolean;
  /** 应用版本，由父组件在启动时取好 */
  appVersion: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "open-help"): void;
  (e: "open-license"): void;
}>();

/** 运行环境信息，打开对话框时填充。 */
const env = ref<{
  tauri: string;
  bundle: string;
  os: string;
  arch: string;
  webview: string;
} | null>(null);

const loading = ref(false);

/** 把 BundleType 枚举值转成用户看得懂的说法。 */
function describeBundle(raw: string): string {
  switch (raw) {
    case "nsis":
      return "exe 安装包";
    case "msi":
      return "msi 安装包";
    case "deb":
      return "deb 包";
    case "rpm":
      return "rpm 包";
    case "app":
      return "应用包";
    case "dmg":
      return "dmg 镜像";
    default:
      return raw || "未知";
  }
}

/** 把 os 插件返回的平台标识转成中文名。 */
function describeOs(type: string, version: string): string {
  const name =
    type === "windows"
      ? "Windows"
      : type === "macos"
        ? "macOS"
        : type === "linux"
          ? "Linux"
          : type;
  return version ? `${name} ${version}` : name;
}

async function loadEnv() {
  if (env.value || loading.value) return;
  loading.value = true;

  try {
    const [{ getTauriVersion, getBundleType }, os] = await Promise.all([
      import("@tauri-apps/api/app"),
      import("@tauri-apps/plugin-os"),
    ]);

    // 各项独立取，任一失败不影响其余——环境信息不该因单个 API 不可用而整体缺失
    const safe = async <T,>(fn: () => Promise<T>, fallback: T): Promise<T> => {
      try {
        return await fn();
      } catch {
        return fallback;
      }
    };

    const [tauri, bundle, osType, osVersion, arch] = await Promise.all([
      safe(() => getTauriVersion(), ""),
      safe(() => getBundleType(), "" as unknown as string),
      safe(() => Promise.resolve(os.type()), ""),
      safe(() => Promise.resolve(os.version()), ""),
      safe(() => Promise.resolve(os.arch()), ""),
    ]);

    env.value = {
      tauri,
      bundle: describeBundle(String(bundle)),
      os: describeOs(osType, osVersion),
      arch,
      // WebView2 版本可从 UA 里取，不额外引入依赖
      webview: detectWebView(),
    };
  } finally {
    loading.value = false;
  }
}

/** 从 User-Agent 里提取 WebView2 / Chromium 版本。 */
function detectWebView(): string {
  const m = navigator.userAgent.match(/(?:Edg|Chrome)\/([\d.]+)/);
  return m?.[1] ? `Chromium ${m[1]}` : "";
}

watch(
  () => props.visible,
  (open) => {
    if (open) void loadEnv();
  },
);
</script>

<template>
  <Transition name="pm-fade">
    <div v-if="visible" class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <button class="close" type="button" title="关闭" @click="emit('close')">
          <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
            <path
              d="M2 2 L10 10 M10 2 L2 10"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
          </svg>
        </button>

        <!-- 头部：图标 + 名称 + 版本 -->
        <div class="head">
          <div class="logo" aria-hidden="true">
            <svg viewBox="0 0 64 64" width="46" height="46">
              <!-- 文档外形 -->
              <path
                d="M14 8 H40 L50 18 V56 H14 Z"
                fill="#ffffff"
                stroke="#c9ccd6"
                stroke-width="2.4"
                stroke-linejoin="round"
              />
              <!-- 折角 -->
              <path d="M40 8 L50 18 H40 Z" fill="#d7dae3" />
              <!-- M 字 -->
              <path
                d="M21 45 V30 L27.5 39 L34 30 V45"
                fill="none"
                stroke="#2b3245"
                stroke-width="4.6"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <!-- 光标 -->
              <rect x="39" y="30" width="4.6" height="15" rx="2.3" fill="#1a7fff" />
              <!-- 文字行 -->
              <rect x="21" y="50" width="20" height="2.6" rx="1.3" fill="#d7dae3" />
            </svg>
          </div>

          <div class="head-text">
            <p class="name">plainmark</p>
            <p class="version">版本 {{ appVersion || "—" }}</p>
          </div>
        </div>

        <p class="tagline">像记事本一样简单的 Markdown 编辑器</p>

        <!-- 运行环境 -->
        <div class="env">
          <div class="env-row">
            <span class="env-key">运行环境</span>
            <span class="env-val">{{ env?.os || "读取中…" }}</span>
          </div>
          <div class="env-row">
            <span class="env-key">架构</span>
            <span class="env-val">{{ env?.arch || "—" }}</span>
          </div>
          <div class="env-row">
            <span class="env-key">安装方式</span>
            <span class="env-val">{{ env?.bundle || "—" }}</span>
          </div>
          <div class="env-row">
            <span class="env-key">Tauri</span>
            <span class="env-val">{{ env?.tauri || "—" }}</span>
          </div>
          <div class="env-row">
            <span class="env-key">渲染引擎</span>
            <span class="env-val">{{ env?.webview || "—" }}</span>
          </div>
        </div>

        <!-- 项目链接 -->
        <div class="links">
          <button type="button" class="link" @click="emit('open-help')">项目主页</button>
          <button type="button" class="link" @click="emit('open-license')">
            MIT 开源协议
          </button>
        </div>

        <p class="copyright">MIT License ｜ © 2026 lanlan0811</p>

        <div class="buttons">
          <button type="button" class="btn btn-primary" @click="emit('close')">确定</button>
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
  z-index: 130;
}

.dialog {
  position: relative;
  width: 400px;
  max-width: calc(100vw - 48px);
  padding: 22px 22px 18px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-lg);
}

.close {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: var(--pm-radius-sm);
  background: transparent;
  color: var(--pm-fg-subtle);
  transition:
    background var(--pm-transition),
    color var(--pm-transition);
}

.close:hover {
  background: var(--pm-chrome-strong);
  color: var(--pm-fg);
}

.head {
  display: flex;
  align-items: center;
  gap: 14px;
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
  width: 58px;
  height: 58px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-chrome);
}

.head-text {
  min-width: 0;
}

.name {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: var(--pm-fg);
  letter-spacing: 0.2px;
}

.version {
  margin: 3px 0 0;
  font-size: 12px;
  color: var(--pm-fg-muted);
}

.tagline {
  margin: 14px 0 0;
  font-size: 12px;
  line-height: 1.6;
  color: var(--pm-fg-muted);
}

/* 运行环境表 */
.env {
  margin: 14px 0 0;
  padding: 10px 12px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-md);
  background: var(--pm-chrome);
}

.env-row {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 3px 0;
  font-size: 11.5px;
}

.env-key {
  flex: none;
  /* 容纳「渲染引擎」四字，避免截断 */
  width: 72px;
  color: var(--pm-fg-subtle);
  white-space: nowrap;
}

.env-val {
  flex: 1;
  min-width: 0;
  color: var(--pm-fg);
  word-break: break-all;
}

/* 项目链接 */
.links {
  display: flex;
  gap: 16px;
  margin: 14px 0 0;
}

.link {
  padding: 0;
  border: none;
  background: transparent;
  color: var(--pm-brand-strong);
  font-size: 12px;
  text-decoration: none;
  border-bottom: 1px solid var(--pm-brand-border);
  transition: border-color var(--pm-transition);
}

.link:hover {
  border-bottom-color: var(--pm-brand);
}

.copyright {
  margin: 12px 0 0;
  font-size: 11px;
  color: var(--pm-fg-subtle);
}

.buttons {
  display: flex;
  justify-content: flex-end;
  margin-top: 18px;
}

.btn {
  min-width: 84px;
  height: 30px;
  padding: 0 14px;
  border-radius: var(--pm-radius-md);
  font-size: 12px;
  transition: background var(--pm-transition);
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
