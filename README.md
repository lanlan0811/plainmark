# plainmark

**简体中文** | [English](README.en.md)

一个极简的 Markdown 桌面编辑器，用起来和 Windows 记事本一样简单。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Platform: Windows 10+](https://img.shields.io/badge/Platform-Windows%2010%2B-blue.svg)

---

## 简介

plainmark 的目标是做**像记事本一样的 Markdown 编辑器**——打开即写，没有侧边栏、没有插件市场、没有账号登录、没有云同步。

大多数 Markdown 编辑器越做越像 IDE，而 plainmark 只关心一件事：让你打开文件、写字、保存，然后关掉。

## 状态

> **早期开发中**，尚未发布任何版本。

目前仓库包含开源协议、CI/CD 流程配置与项目文档，应用代码正在开发。首个版本发布后会在此处更新下载链接。

## 目标特性

- **极简界面**：单窗口，无侧边栏、无标签页、无工具栏堆叠
- **Markdown 语法高亮**：写作时保持源码可读
- **本地优先**：文件全部存在本地，不联网、不上传
- **轻量快速**：启动迅速，占用低
- **开箱即用**：安装后无需任何配置

## 技术栈

| 层 | 选型 |
|---|---|
| 桌面框架 | [Tauri](https://tauri.app/) |
| 前端 | [Vue 3](https://vuejs.org/) |
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

- [贡献指南](CONTRIBUTING.md) —— 开发环境、提交流程、发布规则
- [行为准则](CODE_OF_CONDUCT.md)
- [安全策略](SECURITY.md) —— 如何私下报告漏洞
- [更新日志](CHANGELOG.md)

## 开源协议

[MIT](LICENSE) © 2026 lanlan0811
