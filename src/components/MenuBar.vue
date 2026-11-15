<script setup lang="ts">
/**
 * 自绘菜单栏。
 *
 * 原生 Win32 菜单由系统绘制，外观与记事本完全一致，也无法定制样式。
 * 这里改为应用内自绘：菜单条目与原生命令一一对应，视觉沿用本应用的设计
 * 语言——品牌色、圆角悬停胶囊、带柔和阴影的浮层。
 *
 * 交互约定：
 *   - 点击展开，展开状态下悬停其他菜单直接切换；
 *   - 点击外部或按 Esc 关闭；
 *   - 键盘支持 ↓/↑/Home/End 在菜单项间移动，←/→ 切换相邻菜单。
 *
 * 菜单项的 id 与 App.vue 中的命令分发一一对应。
 */
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";

interface MenuItem {
  /** 菜单命令 id；分隔线没有 id */
  id?: string;
  label?: string;
  /** 右侧显示的快捷键提示，仅作展示，实际快捷键由 App.vue 统一处理 */
  shortcut?: string;
  type?: "item" | "separator" | "checkbox";
  checked?: boolean;
  disabled?: boolean;
}

interface MenuGroup {
  id: string;
  label: string;
  items: MenuItem[];
}

const props = defineProps<{
  wordWrap: boolean;
  statusBar: boolean;
  preview: boolean;
}>();

const emit = defineEmits<{
  (e: "command", id: string): void;
}>();

const menus = computed<MenuGroup[]>(() => [
  {
    id: "file",
    label: "文件",
    items: [
      { id: "file.new", label: "新建", shortcut: "Ctrl+N" },
      { id: "file.new_window", label: "新建窗口", shortcut: "Ctrl+Shift+N" },
      { id: "file.open", label: "打开…", shortcut: "Ctrl+O" },
      { type: "separator" },
      { id: "file.save", label: "保存", shortcut: "Ctrl+S" },
      { id: "file.save_as", label: "另存为…", shortcut: "Ctrl+Shift+S" },
      { type: "separator" },
      { id: "file.exit", label: "退出" },
    ],
  },
  {
    id: "edit",
    label: "编辑",
    items: [
      { id: "edit.undo", label: "撤销", shortcut: "Ctrl+Z" },
      { id: "edit.redo", label: "重做", shortcut: "Ctrl+Y" },
      { type: "separator" },
      { id: "edit.cut", label: "剪切", shortcut: "Ctrl+X" },
      { id: "edit.copy", label: "复制", shortcut: "Ctrl+C" },
      { id: "edit.paste", label: "粘贴", shortcut: "Ctrl+V" },
      { id: "edit.delete", label: "删除", shortcut: "Del" },
      { type: "separator" },
      { id: "edit.find", label: "查找…", shortcut: "Ctrl+F" },
      { id: "edit.find_next", label: "查找下一个", shortcut: "F3" },
      { id: "edit.find_prev", label: "查找上一个", shortcut: "Shift+F3" },
      { id: "edit.replace", label: "替换…", shortcut: "Ctrl+H" },
      { id: "edit.goto", label: "转到…", shortcut: "Ctrl+G" },
      { type: "separator" },
      { id: "edit.select_all", label: "全选", shortcut: "Ctrl+A" },
      { id: "edit.time_date", label: "时间/日期", shortcut: "F5" },
    ],
  },
  {
    id: "format",
    label: "格式",
    items: [
      { id: "format.wordwrap", label: "自动换行", type: "checkbox", checked: props.wordWrap },
      { type: "separator" },
      { id: "format.font", label: "字体…" },
    ],
  },
  {
    id: "view",
    label: "查看",
    items: [
      { id: "view.zoom_in", label: "放大", shortcut: "Ctrl+=" },
      { id: "view.zoom_out", label: "缩小", shortcut: "Ctrl+-" },
      { id: "view.zoom_reset", label: "恢复默认缩放", shortcut: "Ctrl+0" },
      { type: "separator" },
      { id: "view.preview", label: "预览侧边栏", type: "checkbox", checked: props.preview },
      { id: "view.statusbar", label: "状态栏", type: "checkbox", checked: props.statusBar },
    ],
  },
  {
    id: "help",
    label: "帮助",
    items: [
      { id: "help.view", label: "查看帮助" },
      { id: "help.check_update", label: "检查更新…" },
      { type: "separator" },
      { id: "help.about", label: "关于 plainmark" },
    ],
  },
]);

const openId = ref<string | null>(null);
/** 当前菜单是怎么打开的：决定再次点击同一触发器时应关闭还是保持展开。 */
const openSource = ref<"click" | "hover" | "keyboard">("click");
const root = ref<HTMLElement | null>(null);

function triggerEl(id: string): HTMLButtonElement | null {
  return root.value?.querySelector<HTMLButtonElement>(`[data-menu="${id}"]`) ?? null;
}

function panelEl(): HTMLElement | null {
  if (!openId.value) return null;
  // 按菜单 id 精确定位：切换菜单时旧面板可能还处于离开过渡中，
  // 用 `.menu-panel` 取到的第一个节点未必是当前菜单。
  return root.value?.querySelector<HTMLElement>(`[data-menu-panel="${openId.value}"]`) ?? null;
}

function enabledItems(): HTMLButtonElement[] {
  const panel = panelEl();
  if (!panel) return [];
  return Array.from(panel.querySelectorAll<HTMLButtonElement>(".menu-item:not(.is-disabled)"));
}

function openMenu(id: string, focusFirst = false) {
  openId.value = id;
  if (!focusFirst) return;
  openSource.value = "keyboard";
  void nextTick(() => enabledItems()[0]?.focus());
}

function toggleMenu(id: string) {
  if (openId.value === id) {
    // 悬停切换过来的菜单视作「刚打开」：这一次点击只确认展开，不关闭。
    // 否则鼠标移过去时菜单已被 hover 打开，点击反而会把它关掉。
    if (openSource.value === "hover") {
      openSource.value = "click";
      return;
    }
    openId.value = null;
    return;
  }
  openSource.value = "click";
  openMenu(id);
}

/** 展开状态下，悬停其他顶级菜单直接切换。 */
function hoverMenu(id: string) {
  if (openId.value !== null && openId.value !== id) {
    openSource.value = "hover";
    openMenu(id);
  }
}

function selectItem(item: MenuItem) {
  if (!item.id || item.disabled) return;
  openId.value = null;
  emit("command", item.id);
}

function closeAndFocusTrigger(id: string) {
  openId.value = null;
  triggerEl(id)?.focus();
}

function switchMenu(currentId: string, delta: number) {
  const ids = menus.value.map((m) => m.id);
  const index = ids.indexOf(currentId);
  const next = ids[(index + delta + ids.length) % ids.length];
  if (next) openMenu(next, true);
}

function onTriggerKeydown(event: KeyboardEvent, id: string) {
  const ids = menus.value.map((m) => m.id);
  const index = ids.indexOf(id);

  switch (event.key) {
    case "ArrowDown":
    case "Enter":
    case " ":
      event.preventDefault();
      openMenu(id, true);
      break;
    case "ArrowRight": {
      event.preventDefault();
      const next = ids[(index + 1) % ids.length]!;
      if (openId.value !== null) openMenu(next, true);
      else triggerEl(next)?.focus();
      break;
    }
    case "ArrowLeft": {
      event.preventDefault();
      const prev = ids[(index - 1 + ids.length) % ids.length]!;
      if (openId.value !== null) openMenu(prev, true);
      else triggerEl(prev)?.focus();
      break;
    }
    case "Escape":
      openId.value = null;
      break;
    default:
      break;
  }
}

function onPanelKeydown(event: KeyboardEvent, menuId: string) {
  const items = enabledItems();
  if (items.length === 0) return;

  switch (event.key) {
    case "ArrowDown": {
      event.preventDefault();
      const index = items.indexOf(document.activeElement as HTMLButtonElement);
      items[(index + 1) % items.length]?.focus();
      break;
    }
    case "ArrowUp": {
      event.preventDefault();
      const index = items.indexOf(document.activeElement as HTMLButtonElement);
      items[(index - 1 + items.length) % items.length]?.focus();
      break;
    }
    case "Home":
      event.preventDefault();
      items[0]?.focus();
      break;
    case "End":
      event.preventDefault();
      items[items.length - 1]?.focus();
      break;
    case "Escape":
      event.preventDefault();
      closeAndFocusTrigger(menuId);
      break;
    case "ArrowRight":
      event.preventDefault();
      switchMenu(menuId, 1);
      break;
    case "ArrowLeft":
      event.preventDefault();
      switchMenu(menuId, -1);
      break;
    default:
      break;
  }
}

function onDocumentPointerDown(event: PointerEvent) {
  if (!openId.value) return;
  if (root.value && !root.value.contains(event.target as Node)) openId.value = null;
}

function onDocumentFocusIn(event: FocusEvent) {
  if (!openId.value) return;
  if (root.value && !root.value.contains(event.target as Node)) openId.value = null;
}

/** 顶级菜单的 Alt 助记键，沿用 Windows 应用的 Alt+F/E/O/V/H 习惯。 */
const ALT_KEYS: Record<string, string> = {
  f: "file",
  e: "edit",
  o: "format",
  v: "view",
  h: "help",
};

/** 鼠标打开菜单时焦点仍留在编辑器里，Esc 与方向键需要在窗口级兜底。 */
function onWindowKeydown(event: KeyboardEvent) {
  // Alt + 助记键：无论菜单是否展开，都直接切到对应菜单
  if (event.altKey && !event.ctrlKey && !event.metaKey) {
    const menuId = ALT_KEYS[event.key.toLowerCase()];
    if (menuId) {
      event.preventDefault();
      openMenu(menuId, true);
    }
    return;
  }

  if (!openId.value) return;

  if (event.key === "Escape") {
    openId.value = null;
    return;
  }

  // 菜单已展开但焦点不在菜单内（鼠标点开的情形）：
  //   - 方向键把焦点接进菜单，之后由面板自身的 keydown 接管；
  //   - 其余按键先关闭菜单。可打印字符与回车同时拦掉默认行为，
  //     避免这次的按键被输入到背后的文档里。
  const panel = panelEl();
  if (!panel || !panel.contains(document.activeElement)) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const items = enabledItems();
      const target = event.key === "ArrowDown" ? items[0] : items[items.length - 1];
      target?.focus();
      return;
    }

    // 修饰键本身不构成一次操作，忽略
    if (event.key === "Shift" || event.key === "Control" || event.key === "Alt" || event.key === "Meta") {
      return;
    }

    const plain = !event.ctrlKey && !event.altKey && !event.metaKey;
    if (plain && (event.key.length === 1 || event.key === "Enter")) {
      event.preventDefault();
    }
    openId.value = null;
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocumentPointerDown, true);
  document.addEventListener("focusin", onDocumentFocusIn, true);
  window.addEventListener("keydown", onWindowKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", onDocumentPointerDown, true);
  document.removeEventListener("focusin", onDocumentFocusIn, true);
  window.removeEventListener("keydown", onWindowKeydown);
});
</script>

<template>
  <div ref="root" class="menubar" role="menubar" aria-label="主菜单">
    <div v-for="menu in menus" :key="menu.id" class="menu-root">
      <button
        class="menu-trigger"
        :class="{ 'is-open': openId === menu.id }"
        type="button"
        role="menuitem"
        aria-haspopup="menu"
        :aria-expanded="openId === menu.id"
        :data-menu="menu.id"
        @mousedown.prevent
        @click="toggleMenu(menu.id)"
        @mouseenter="hoverMenu(menu.id)"
        @keydown="onTriggerKeydown($event, menu.id)"
      >
        {{ menu.label }}
      </button>

      <Transition name="pm-menu">
        <div
          v-if="openId === menu.id"
          class="menu-panel"
          :class="{ 'has-checks': menu.items.some((i) => i.type === 'checkbox') }"
          role="menu"
          :aria-label="menu.label"
          :data-menu-panel="menu.id"
          @mousedown.prevent
          @keydown="onPanelKeydown($event, menu.id)"
        >
          <template v-for="(item, index) in menu.items" :key="item.id ?? `sep-${index}`">
            <div v-if="item.type === 'separator'" class="menu-separator" role="separator"></div>
            <button
              v-else
              class="menu-item"
              :class="{ 'is-disabled': item.disabled }"
              type="button"
              :role="item.type === 'checkbox' ? 'menuitemcheckbox' : 'menuitem'"
              :aria-checked="item.type === 'checkbox' ? item.checked : undefined"
              :disabled="item.disabled"
              tabindex="-1"
              @click="selectItem(item)"
            >
              <span class="check" aria-hidden="true">
                <svg v-if="item.type === 'checkbox' && item.checked" viewBox="0 0 10 10" width="10" height="10">
                  <path
                    d="M1.5 5 L4 7.4 L8.5 2.2"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.7"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </span>
              <span class="label">{{ item.label }}</span>
              <span v-if="item.shortcut" class="shortcut">{{ item.shortcut }}</span>
            </button>
          </template>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.menubar {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  min-width: 0;
}

.menu-root {
  position: relative;
  flex: none;
}

.menu-trigger {
  height: 22px;
  padding: 0 10px;
  border: none;
  border-radius: var(--pm-radius-sm);
  background: transparent;
  color: var(--pm-fg-muted);
  font-family: inherit;
  font-size: 12px;
  line-height: 1;
  white-space: nowrap;
  transition:
    background var(--pm-transition),
    color var(--pm-transition);
}

.menu-trigger:hover {
  background: var(--pm-chrome-strong);
  color: var(--pm-fg);
}

.menu-trigger.is-open {
  background: var(--pm-brand-soft);
  color: var(--pm-brand-strong);
  font-weight: 500;
}

/* 浮层：白底圆角卡片，与对话框保持同一套语言。
   统一从触发器左缘展开——菜单栏不在窗口最右侧，右对齐会让浮层压住相邻菜单。 */
.menu-panel {
  position: absolute;
  top: calc(100% + 7px);
  left: 0;
  z-index: 200;
  min-width: 232px;
  max-width: calc(100vw - 16px);
  max-height: calc(100vh - 56px);
  overflow-y: auto;
  padding: 5px;
  border: 1px solid var(--pm-border);
  border-radius: var(--pm-radius-lg);
  background: var(--pm-bg);
  box-shadow: var(--pm-shadow-md);
}

.menu-item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 4px;
  width: 100%;
  height: 30px;
  padding: 0 10px;
  border: none;
  border-radius: var(--pm-radius-sm);
  background: transparent;
  color: var(--pm-fg);
  font-family: inherit;
  font-size: 12px;
  text-align: left;
  white-space: nowrap;
  transition:
    background var(--pm-transition),
    color var(--pm-transition);
}

/* 含勾选项的菜单才为勾选标记留出左侧一列，其余菜单保持紧凑的对齐 */
.menu-panel.has-checks .menu-item {
  grid-template-columns: 18px minmax(0, 1fr) auto;
  padding-left: 4px;
}

.menu-item:hover,
.menu-item:focus {
  background: var(--pm-brand-soft);
  color: var(--pm-brand-strong);
  outline: none;
}

.menu-item.is-disabled {
  color: var(--pm-fg-subtle);
  cursor: default;
}

.menu-item.is-disabled:hover,
.menu-item.is-disabled:focus {
  background: transparent;
  color: var(--pm-fg-subtle);
}

.check {
  display: none;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  color: var(--pm-brand);
}

.menu-panel.has-checks .check {
  display: flex;
}

.label {
  overflow: hidden;
  text-overflow: ellipsis;
}

.shortcut {
  color: var(--pm-fg-subtle);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.1px;
}

.menu-item:hover .shortcut,
.menu-item:focus .shortcut {
  color: var(--pm-brand);
  opacity: 0.75;
}

.menu-separator {
  height: 1px;
  margin: 4px 6px;
  background: var(--pm-border);
}

.pm-menu-enter-active,
.pm-menu-leave-active {
  transition:
    opacity 100ms ease,
    transform 100ms ease;
  transform-origin: top left;
}

.pm-menu-enter-from,
.pm-menu-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.98);
}
</style>
