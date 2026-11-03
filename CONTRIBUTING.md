# 贡献指南

感谢你对 plainmark 的关注。本文档说明如何参与贡献。

## 行为准则

参与本项目即表示你同意遵守[行为准则](CODE_OF_CONDUCT.md)。

## 项目定位

plainmark 的目标是**像记事本一样的 Markdown 编辑器**。提交功能建议前，请先确认它符合这个定位：

- ✅ 让写作更简单、更直接的功能
- ❌ 侧边栏、插件系统、账号体系、云同步等重型特性

功能取舍上，「像记事本」优先于「功能全」。

## 报告问题

请使用 [Issue 模板](https://github.com/lanlan0811/plainmark/issues/new/choose) 提交，并尽量提供：

- **Bug 报告**：复现步骤、期望行为、实际行为、系统版本、plainmark 版本
- **功能建议**：使用场景、为什么现有方式不够用、是否契合项目定位

提交前请先搜索已有 Issue，避免重复。

## 开发环境

### 环境要求

- [Node.js](https://nodejs.org/) 20 或更高版本
- [Rust](https://www.rust-lang.org/tools/install) stable 工具链
- Windows 10 及以上

### 本地运行

```bash
npm ci                # 安装前端依赖（请用 ci，不要用 install）
npm run tauri dev     # 开发模式，支持热重载
```

### 提交前自检

```bash
npm run build         # 前端构建
cargo check --manifest-path src-tauri/Cargo.toml   # Rust 编译检查
```

## 提交流程

1. Fork 本仓库并从 `main` 创建分支，命名如 `fix/xxx`、`feat/xxx`
2. 完成修改，确保上述自检命令通过
3. 提交信息使用**中文**，简明描述「为什么改」而非「改了什么」
4. 向 `main` 发起 Pull Request，在描述中说明改动动机与验证方式

### 代码风格

- 格式化交给工具，不要手动调整：`cargo fmt`、前端沿用项目现有风格
- 换行符统一为 `LF`，由 `.gitattributes` 自动处理
- 不引入与本项目定位无关的依赖

## 发布规则

正式版本由 GitHub Actions 自动构建、打包与发布，**不接受本地构建的安装包**。

推送 `v*` 格式的 tag 会触发完整流程：

```
push tag v* → Build（构建 + 打包） → 全绿 → Release（自动发版）
                                       └ 失败或取消 → 不发版
```

| 环节 | 说明 |
|---|---|
| 触发 | 仅推送 `v*` tag 时构建并可能发版；手动触发只构建、不发版 |
| 构建 | `windows-latest`，`npm ci` + `npm run tauri build` |
| 打包 | MSI 与 NSIS 两种安装包必须同时产出，缺一即失败 |
| 发版门控 | 构建与打包全绿、由 tag 推送触发、tag 以 `v` 开头，三者全满足才发版 |
| 产物复用 | 发版不重新构建，直接复用构建阶段的产物 |

版本号遵循[语义化版本](https://semver.org/lang/zh-CN/) `vMAJOR.MINOR.PATCH`，tag 名必须与 `src-tauri/tauri.conf.json` 中的 `version` 一致。tag 一经推送即对外发布，不得删除或重打；如需修正请递增 PATCH 号重新发布。

**维护者注意**：请勿移除发版 workflow 中的全绿门控条件，也勿使用 `continue-on-error` 绕过构建失败。

## 开源协议

本项目采用 [MIT 协议](LICENSE)。提交贡献即表示你同意以该协议授权你的作品。
