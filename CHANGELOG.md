# 更新日志

本项目的所有重要变更都会记录在此文件中。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循[语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

- **应用本体**：基于 Tauri 2 + Vue 3 + Rust 的 Markdown 编辑器
  - 原生菜单栏，结构对齐 Windows 记事本：文件 / 编辑 / 格式 / 查看 / 帮助
  - 标签页式多文档，每标签独立撤销历史
  - CodeMirror 6 编辑器内核，外观对齐记事本（无行号、无活动行高亮、制表位 8）
  - Markdown 语法高亮（纯文本编辑，无预览面板）
  - 编码支持：ANSI / UTF-8 / UTF-8 BOM / UTF-16 LE / UTF-16 BE，打开时自动检测、保存时原样写回
  - 换行符支持：Windows (CRLF) / Unix (LF) / Macintosh (CR)，状态栏可切换
  - 状态栏显示行、列、缩放、换行符、编码
  - 未保存提示（保存 / 不保存 / 取消）、外部改动检测、拖放打开、命令行传参打开
  - 单实例运行，第二次启动在已有窗口开新标签
- **Windows Shell 集成**：关联 `.md` / `.markdown`，并在右键「新建」菜单加入「Markdown 文档」
  - NSIS 通过 `windows/hooks.nsh` 写入 `ShellNew` 键
  - MSI 通过 `windows/fragments/shellnew.wxs` 写入同样的键
- **项目基建**：MIT 协议、中英文 README、CI/CD（构建打包 + tag 门控发版）、开源协作文档、应用图标

### 说明

首个正式版本尚未发布。发布后本节将替换为带版本号的正式记录。
