# plainmark 构建、打包与发布规则

本文档说明 plainmark 的 CI/CD 规则。所有构建、打包、发版均由 GitHub Actions 自动完成，**本地不产出正式发布物**。

---

## 1. 总览

```
                 ┌─────────────────────────────┐
  push tag v* ──▶│  Build（构建 + 打包）        │
                 │  windows-latest             │
                 └──────────────┬──────────────┘
                                │
                   conclusion == 'success' ?
                                │
                    ┌───────────┴───────────┐
                   是 │                       │ 否（失败/取消）
                    ▼                       ▼
        ┌───────────────────────┐   ┌──────────────────┐
        │ Release（发版）        │   │ 不发版，流程终止  │
        │ ubuntu-latest         │   └──────────────────┘
        └───────────────────────┘
```

**核心原则：构建与打包全绿，才允许发版。** 任一环节失败或取消，发版 job 被整体跳过。

涉及两个 workflow 文件：

| 文件 | 职责 | 运行环境 |
|---|---|---|
| `.github/workflows/build.yml` | 构建 + 打包 + 上传产物 | `windows-latest` |
| `.github/workflows/release.yml` | 发布 GitHub Release | `ubuntu-latest` |

---

## 2. 触发规则

| 事件 | 是否触发 Build | 是否发版 |
|---|---|---|
| 推送 `v*` tag（如 `v0.1.0`） | ✅ | ✅ 条件满足时 |
| 手动 `workflow_dispatch` | ✅ | ❌ 永不发版 |
| 推送普通分支 / PR | ❌ | ❌ |

**规则：**

- 发版**只能**通过推送 tag 触发，不存在其他入口。
- 手动触发仅用于验证构建是否通过，不产生 Release。
- 普通分支推送不会触发任何 CI，避免浪费 Actions 配额。

---

## 3. 构建与打包规则（`build.yml`）

### 3.1 环境

| 项目 | 版本 / 配置 |
|---|---|
| 运行环境 | `windows-latest`（对应 Windows Server 2022） |
| Node.js | 20（启用 npm 缓存） |
| Rust | stable（`dtolnay/rust-toolchain`） |
| Rust 缓存 | `swatinem/rust-cache`，工作目录 `src-tauri` |
| 前端依赖安装 | `npm ci`（严格按 `package-lock.json`，禁止 `npm install`） |

### 3.2 构建命令

```bash
npm run tauri build
```

该命令同时完成前端构建（Vue）与 Rust 编译，并产出安装包。**不允许在 CI 中拆分执行前端与 Rust 构建**，以保证产物一致性。

### 3.3 打包产物

| 产物 | 路径 | 说明 |
|---|---|---|
| MSI 安装包 | `src-tauri/target/release/bundle/msi/*.msi` | Windows Installer，适合企业分发 |
| NSIS 安装包 | `src-tauri/target/release/bundle/nsis/*.exe` | 轻量安装程序，适合个人用户 |

**规则：**

- 两种安装包**必须同时产出**，缺一即视为打包失败。
- `if-no-files-found: error` —— 未匹配到任何产物时 job 直接失败，防止「假绿」。
- Artifact 名称固定为 `plainmark-windows`，保留 **7 天**。
- 目标平台为 Windows 10 及以上（由 `src-tauri/tauri.conf.json` 的 `bundle.targets` 与 WebView2 策略决定）。

### 3.4 并发控制

```yaml
concurrency:
  group: build-${{ github.ref }}
  cancel-in-progress: true
```

同一 tag 重复触发时，旧的一轮自动取消，只保留最新一次构建。

---

## 4. 发布规则（`release.yml`）

### 4.1 门控条件

发版 job 必须**同时**满足以下三个条件，缺一不发版：

| # | 条件 | 含义 |
|---|---|---|
| 1 | `workflow_run.conclusion == 'success'` | **Build 全绿**——构建或打包任一步失败/取消则不发版 |
| 2 | `workflow_run.event == 'push'` | 排除手动 dispatch，防止手动构建误发 Release |
| 3 | `startsWith(workflow_run.head_branch, 'v')` | 仅 `v*` tag 可发版，挡住分支推送 |

### 4.2 产物来源

发版**不重新构建**，而是通过 `run-id` 拉取对应那轮 Build 已上传的 artifact：

```yaml
run-id: ${{ github.event.workflow_run.id }}
```

**规则：发版与构建使用同一份二进制产物**，保证「测试过的就是发布的」，杜绝二次构建引入差异。

### 4.3 Release 配置

| 配置项 | 值 | 说明 |
|---|---|---|
| `tag_name` | 触发构建的 tag 名 | 如 `v0.1.0` |
| `target_commitish` | 该轮构建的 commit SHA | 锁定到确切提交 |
| `generate_release_notes` | `true` | 自动生成变更日志 |
| `draft` | `false` | 直接正式发布 |
| `prerelease` | `false` | 非预发布 |
| 附件 | `dist/**/*` | MSI + NSIS 安装包 |

### 4.4 权限

```yaml
permissions:
  contents: write   # 发版必需
```

`build.yml` 使用最小权限 `contents: read`，仅 `release.yml` 持有写权限。

---

## 5. 版本号规则

采用[语义化版本](https://semver.org/lang/zh-CN/) `vMAJOR.MINOR.PATCH`：

| 变更类型 | 版本递增 | 示例 |
|---|---|---|
| 不兼容的破坏性变更 | MAJOR | `v1.0.0` → `v2.0.0` |
| 向后兼容的新功能 | MINOR | `v1.0.0` → `v1.1.0` |
| 向后兼容的缺陷修复 | PATCH | `v1.0.0` → `v1.0.1` |

**规则：**

- tag 名**必须**以 `v` 开头，否则不会触发 Build，更不会发版。
- tag 名与 `src-tauri/tauri.conf.json` 中的 `version` 字段**必须一致**。
- tag 一经推送即视为对外发布，**不得删除或重打**（如需修正请递增 PATCH 号重发）。

---

## 6. 标准发版流程

```bash
# 1. 确认工作区干净，且已在 main 分支
git status
git checkout main && git pull

# 2. 同步版本号（tauri.conf.json 与 package.json）
#    修改 src-tauri/tauri.conf.json -> "version": "0.1.0"

# 3. 提交版本变更
git add src-tauri/tauri.conf.json package.json
git commit -m "chore: 发布 v0.1.0"
git push origin main

# 4. 打 tag 并推送 —— 这一步才真正触发 CI
git tag v0.1.0
git push origin v0.1.0
```

推送 tag 后：

1. `Build` workflow 自动启动，在 `windows-latest` 上构建并打包。
2. 构建与打包全绿后，`Release` workflow 被触发，自动创建 GitHub Release 并附上 MSI 与 NSIS 安装包。
3. 若任一步失败，Release 不会执行，需修复后**递增版本号**重新走流程。

---

## 7. 失败处理

| 失败环节 | 现象 | 处理方式 |
|---|---|---|
| 前端构建失败 | Build job 红，无 artifact | 本地 `npm run build` 复现并修复 |
| Rust 编译失败 | Build job 红，无 artifact | 本地 `cargo check` 复现并修复 |
| 打包无产物 | Build job 红（`if-no-files-found: error`） | 检查 `tauri.conf.json` 的 `bundle.targets` 是否包含 `msi`、`nsis` |
| Build 失败后 | Release 未执行 | 属预期行为，**不发版即正确** |
| Release 失败 | Build 绿但无 Release | 检查 tag 是否以 `v` 开头、是否为 tag 推送触发 |

**规则：禁止通过放宽门控条件（如去掉 `conclusion == 'success'`）来「让流程跑通」。** 门控是发布质量的唯一保障。

---

## 8. 禁止事项

- ❌ 手动在本地上传安装包到 Release —— 发布物只能来自 CI。
- ❌ 在 CI 中跳过测试或忽略构建错误（`--no-verify`、`continue-on-error` 等）。
- ❌ 复用已发布的 tag 重新发版。
- ❌ 在 Build 阶段就创建 Release（必须由 `release.yml` 统一负责）。
- ❌ 修改 workflow 时移除 `conclusion == 'success'` 门控。

---

## 9. 相关文件

| 文件 | 说明 |
|---|---|
| `.github/workflows/build.yml` | 构建与打包 |
| `.github/workflows/release.yml` | 发版 |
| `src-tauri/tauri.conf.json` | 版本号、打包目标、应用元信息 |
| `package.json` | 前端版本号与构建脚本 |
