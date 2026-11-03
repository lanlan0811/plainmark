//! plainmark 应用入口。

mod commands;
mod io;
mod menu;
mod state;

use std::path::PathBuf;

use tauri::{Emitter, Manager};

use state::AppState;

/// 从前端接收菜单事件时使用的通道名。
const MENU_EVENT: &str = "plainmark://menu";

/// 从命令行参数里挑出看起来是文件路径的项。
///
/// Windows 上传入的路径带引号，`args_os` 已由系统完成解引号。这里只保留
/// 真实存在的文件，避免把 tauri 的开发参数误当成文件。
fn file_args() -> Vec<String> {
    std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动时先把命令行里的文件记下来，供前端初始化时取用
    let startup_files = file_args();

    let mut builder = tauri::Builder::default();

    // 单实例插件必须第一个注册，否则第二次启动不会走到它的回调
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // 第二次启动：聚焦已有窗口，并把参数中的文件交给前端开成新标签
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }

            let files: Vec<String> = args
                .iter()
                .skip(1)
                .map(PathBuf::from)
                .filter(|p| p.is_file())
                .map(|p| p.to_string_lossy().into_owned())
                .collect();

            if !files.is_empty() {
                // 前端监听这个事件，收到后逐个开成标签
                let _ = app.emit("plainmark://open-files", files);
            }
        }));
    }

    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::read_document,
            commands::write_document,
            commands::get_startup_files,
            commands::set_window_dirty,
            commands::open_help,
            commands::close_window,
            commands::set_title,
            commands::file_modified_ms,
            commands::exit_app,
        ])
        .setup(move |app| {
            menu::build(app.handle())?;

            let state = app.state::<AppState>();
            *state.startup_files.lock().unwrap() = startup_files.clone();

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().0.as_str();

            // 只有「新建窗口」完全在 Rust 侧处理；其余（含退出）都转给前端，
            // 因为退出需要先走前端的未保存提示流程。
            if id == "file.new_window" {
                if let Err(e) = menu::open_new_window(app) {
                    eprintln!("新建窗口失败：{e}");
                }
                return;
            }

            // 菜单事件不带窗口 label，转发给当前聚焦的窗口
            let target = app
                .webview_windows()
                .into_iter()
                .find(|(_, w)| w.is_focused().unwrap_or(false))
                .map(|(label, _)| label)
                .unwrap_or_else(|| "main".to_string());

            let _ = app.emit_to(target, MENU_EVENT, MenuEventPayload { id: id.to_string() });
        })
        .run(tauri::generate_context!())
        .expect("启动 plainmark 失败");
}

/// 转发给前端的菜单事件载荷。
#[derive(Clone, serde::Serialize)]
struct MenuEventPayload {
    id: String,
}
