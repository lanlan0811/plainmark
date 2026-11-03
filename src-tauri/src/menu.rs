//! 原生菜单栏的构建。
//!
//! 结构对齐 Windows 记事本：文件(F) / 编辑(E) / 格式(O) / 查看(V) / 帮助(H)。
//! 菜单项的 id 用点号分段（如 `file.new`），前端按 id 分发具体行为。
//!
//! 按产品决策，**不包含**打印与页面设置。
//!
//! 这里固定使用默认的 Wry 运行时，不做运行时泛型——桌面端应用没有多运行时需求，
//! 具体类型也更不容易出编译问题。

use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager};

/// 勾选类菜单项的 id。勾选状态由平台侧自动切换，前端只负责同步自身状态。
pub const ID_WORD_WRAP: &str = "format.wordwrap";
pub const ID_STATUS_BAR: &str = "view.statusbar";
pub const ID_PREVIEW: &str = "view.preview";

/// 构建并设置应用菜单。
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    // ---- 文件(F) ----
    let file = SubmenuBuilder::new(app, "文件(F)")
        .item(&MenuItemBuilder::with_id("file.new", "新建(N)").accelerator("Ctrl+N").build(app)?)
        .item(
            &MenuItemBuilder::with_id("file.new_window", "新建窗口(W)")
                .accelerator("Ctrl+Shift+N")
                .build(app)?,
        )
        .item(&MenuItemBuilder::with_id("file.open", "打开(O)...").accelerator("Ctrl+O").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("file.save", "保存(S)").accelerator("Ctrl+S").build(app)?)
        .item(
            &MenuItemBuilder::with_id("file.save_as", "另存为(A)...")
                .accelerator("Ctrl+Shift+S")
                .build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("file.exit", "退出(X)").build(app)?)
        .build()?;

    // ---- 编辑(E) ----
    let edit = SubmenuBuilder::new(app, "编辑(E)")
        .item(&MenuItemBuilder::with_id("edit.undo", "撤销(U)").accelerator("Ctrl+Z").build(app)?)
        .item(&MenuItemBuilder::with_id("edit.redo", "重做(R)").accelerator("Ctrl+Y").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("edit.cut", "剪切(T)").accelerator("Ctrl+X").build(app)?)
        .item(&MenuItemBuilder::with_id("edit.copy", "复制(C)").accelerator("Ctrl+C").build(app)?)
        .item(&MenuItemBuilder::with_id("edit.paste", "粘贴(P)").accelerator("Ctrl+V").build(app)?)
        .item(&MenuItemBuilder::with_id("edit.delete", "删除(L)").accelerator("Del").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("edit.find", "查找(F)...").accelerator("Ctrl+F").build(app)?)
        .item(&MenuItemBuilder::with_id("edit.find_next", "查找下一个(N)").accelerator("F3").build(app)?)
        .item(
            &MenuItemBuilder::with_id("edit.find_prev", "查找上一个(V)")
                .accelerator("Shift+F3")
                .build(app)?,
        )
        .item(&MenuItemBuilder::with_id("edit.replace", "替换(R)...").accelerator("Ctrl+H").build(app)?)
        .item(&MenuItemBuilder::with_id("edit.goto", "转到(G)...").accelerator("Ctrl+G").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("edit.select_all", "全选(A)").accelerator("Ctrl+A").build(app)?)
        .item(
            &MenuItemBuilder::with_id("edit.time_date", "时间/日期(D)")
                .accelerator("F5")
                .build(app)?,
        )
        .build()?;

    // ---- 格式(O) ----
    let word_wrap = CheckMenuItemBuilder::with_id(ID_WORD_WRAP, "自动换行(W)")
        .checked(false)
        .build(app)?;
    let format = SubmenuBuilder::new(app, "格式(O)")
        .item(&word_wrap)
        .item(&MenuItemBuilder::with_id("format.font", "字体(F)...").build(app)?)
        .build()?;

    // ---- 查看(V) ----
    let status_bar = CheckMenuItemBuilder::with_id(ID_STATUS_BAR, "状态栏(S)")
        .checked(true)
        .build(app)?;
    let zoom = SubmenuBuilder::new(app, "缩放(Z)")
        .item(
            &MenuItemBuilder::with_id("view.zoom_in", "放大(I)")
                .accelerator("Ctrl+Plus")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("view.zoom_out", "缩小(O)")
                .accelerator("Ctrl+-")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("view.zoom_reset", "恢复默认缩放(R)")
                .accelerator("Ctrl+0")
                .build(app)?,
        )
        .build()?;
    let preview = CheckMenuItemBuilder::with_id(ID_PREVIEW, "预览侧边栏(P)")
        .checked(true)
        .build(app)?;
    let view = SubmenuBuilder::new(app, "查看(V)")
        .item(&zoom)
        .separator()
        .item(&preview)
        .item(&status_bar)
        .build()?;

    // ---- 帮助(H) ----
    let help = SubmenuBuilder::new(app, "帮助(H)")
        .item(&MenuItemBuilder::with_id("help.view", "查看帮助(H)").build(app)?)
        .item(&MenuItemBuilder::with_id("help.check_update", "检查更新(U)...").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("help.about", "关于 plainmark(A)").build(app)?)
        .build()?;

    let menu = MenuBuilder::new(app)
        .items(&[&file, &edit, &format, &view, &help])
        .build()?;

    app.set_menu(menu)?;

    Ok(())
}

/// 打开一个新窗口。label 依次为 main、main-2、main-3…
///
/// 新窗口的尺寸会参照当前主窗口，而不是用固定值——用户调整过主窗口大小后，
/// 新窗口应当与之协调。
pub fn open_new_window(app: &AppHandle) -> tauri::Result<()> {
    // 找一个尚未占用的 label
    let mut index = 1;
    let label = loop {
        let candidate = if index == 1 {
            "main".to_string()
        } else {
            format!("main-{index}")
        };
        if app.get_webview_window(&candidate).is_none() {
            break candidate;
        }
        index += 1;
    };

    // 新窗口沿用当前主窗口的尺寸，避免用户调整过大小后新窗口又回到默认值。
    // 取不到时（主窗口已关闭）退回一个保守的默认值。
    let (width, height) = app
        .get_webview_window("main")
        .and_then(|w| w.inner_size().ok())
        .map(|s| {
            let scale = app
                .get_webview_window("main")
                .and_then(|w| w.scale_factor().ok())
                .unwrap_or(1.0);
            (s.width as f64 / scale, s.height as f64 / scale)
        })
        .unwrap_or((900.0, 620.0));

    // WebviewUrl 没有 Default，必须显式指向入口页面
    let url = tauri::WebviewUrl::App("index.html".into());
    tauri::WebviewWindowBuilder::new(app, &label, url)
        .title("无标题 - plainmark")
        .inner_size(width, height)
        .min_inner_size(480.0, 320.0)
        .center()
        .build()?;

    Ok(())
}
