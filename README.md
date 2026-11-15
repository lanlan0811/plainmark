# plainmark

**简体中文** | [English](README.en.md)

一个极简的 Markdown 桌面编辑器，用起来和 Windows 记事本一样简单。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Platform: Windows 10+](https://img.shields.io/badge/Platform-Windows%2010%2B-blue.svg)
[![Release](https://img.shields.io/github/v/release/lanlan0811/plainmark)](https://github.com/lanlan0811/plainmark/releases/latest)

---

## 简介

plainmark 的目标是做**像记事本一样的 Markdown 编辑器**——打开即写，没有文件树、没有插件市场、没有账号登录、没有云同步。

大多数 Markdown 编辑器越做越像 IDE，而 plainmark 只关心一件事：让你打开文件、写字、保存，然后关掉。

## 下载

从 [Releases](https://github.com/lanlan0811/plainmark/releases/latest) 页面下载最新版：

| 安装包 | 适用场景 |
|---|---|
| `plainmark_<版本>_x64-setup.exe` | NSIS 安装程序，体积小，推荐个人用户 |
| `plainmark_<版本>_x64_zh-CN.msi` | MSI 安装包，适合企业批量分发 |

两者均支持 **Windows 10 及以上**。已安装旧版本的用户可通过「帮助 → 检查更新」自动升级，无需手动下载。

## 特性

**像记事本**

- 原生菜单栏（文件 / 编辑 / 格式 / 查看 / 帮助）与状态栏，布局和快捷键对齐 Windows 记事本
- 状态栏显示行列位置、缩放、换行符与编码，可点击的字段即点即改
- 未保存提示（保存 / 不保存 / 取消）、外部改动检测、拖放打开、命令行传参打开
- 单实例运行：第二次启动会在已有窗口里开新标签
- 记住窗口尺寸与位置，下次启动恢复（含最大化）

**写作**

- **Markdown 语法高亮**：只给源码上色，编辑面始终是纯文本，不打断写作
- **标签页**：一个窗口开多个文档，每个标签独立撤销历史
- **可视化预览侧边栏**：右侧预览，可调宽（260–900px）、可随时开关，宽度与开关状态会记住
  - 支持 GitHub 风格表格、任务列表、删除线与裸链接自动识别
  - 渲染结果经 DOMPurify 清洗，脚本与事件属性一律拦截
  - **拖拽重排**：拖动预览中的块改变顺序，Markdown 源码同步更新
- **图片拖入**：把图片拖进窗口即复制到文档同目录的 `assets/` 下，重名自动加序号，并插入相对路径

**文件**

- **编码无损**：ANSI / UTF-8 / UTF-8 BOM / UTF-16 LE / UTF-16 BE 自动识别，保存时原样写回
- **换行符可切**：Windows (CRLF) / Unix (LF) / Macintosh (CR)，状态栏一键切换
- **本地优先**：文件全部存在本地，不联网、不上传
- **右键新建**：安装后在资源管理器右键「新建」菜单直接创建 Markdown 文档

## 技术栈

| 层 | 选型 |
|---|---|
| 桌面框架 | [Tauri](https://tauri.app/) |
| 前端 | [Vue 3](https://vuejs.org/) |
| 编辑器 | [CodeMirror 6](https://codemirror.net/) |
| 预览渲染 | [markdown-it](https://github.com/markdown-it/markdown-it) + [DOMPurify](https://github.com/cure53/DOMPurify) |
| 后端 | Rust |
| 目标平台 | Windows 10 及以上 |

选择 Tauri 而非 Electron，是为了更小的安装体积和更低的运行内存占用，贴合「轻量记事本」的定位。

## 构建

### 环境要求

- [Node.js](https://nodejs.org/) 20 或更高版本
- [Rust](https://www.rust-lang.org/tools/install) stable 工具链
- Windows 10 及以上（需 WebView2 运行时，Windows 11 已内置）

### 本地构建

```bash
# 安装前端依赖
npm ci

# 开发模式（热重载）
npm run tauri dev

# 构建并打包安装程序
npm run tauri build
```

打包产物位于 `src-tauri/target/release/bundle/`，包含 MSI 与 NSIS 两种安装包。

## 发布

正式版本的构建、打包与发布**全部由 GitHub Actions 自动完成**，本地不产出发布物。

推送一个 `v*` 格式的 tag 即可触发完整流程：

```bash
git tag v0.1.0
git push origin v0.1.0
```

构建与打包全绿后，会自动创建 GitHub Release 并附上安装包。详细规则见 [CONTRIBUTING.md](CONTRIBUTING.md#发布规则)。

## 贡献

项目处于早期阶段，接口与结构可能频繁变动。欢迎通过 Issue 提出建议或报告问题。

- [贡献指南](CONTRIBUTING.md)（[English](CONTRIBUTING.en.md)）—— 开发环境、提交流程、发布规则
- [行为准则](CODE_OF_CONDUCT.md)
- [安全策略](SECURITY.md) —— 如何私下报告漏洞
- [更新日志](CHANGELOG.md)

## 已知限制

- 打印与页面设置尚未实现，菜单中不提供
- 字体对话框为自建界面（WebView2 无原生字体选择器），与记事本存在细微差异
- 查找 / 替换使用编辑器内置面板，尚未做成记事本的中文独立对话框
- 不接管 `.txt` 关联，避免抢占记事本的默认地位
- 预览中的远程图片（如 shields.io 徽章）会联网加载
- 启动速度受限于 WebView2 运行时初始化，无法与原生记事本完全一致

## 开源协议

[MIT](LICENSE) © 2026 lanlan0811
