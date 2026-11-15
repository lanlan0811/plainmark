//! 多窗口管理。
//!
//! 菜单栏已改为应用内自绘（见 `src/components/MenuBar.vue`），Rust 侧不再
//! 构建原生菜单；这里只保留「新建窗口」这一需要后端创建 WebviewWindow 的能力。

use tauri::{AppHandle, Manager};

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
