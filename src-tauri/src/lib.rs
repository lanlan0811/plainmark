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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            // 记住窗口尺寸与位置，下次启动恢复。
            //
            // 刻意不包含 StateFlags::VISIBLE：窗口的可见性由「首屏渲染完成后再显示」
            // 这一启动优化控制（见 tauri.conf.json 的 visible: false 与 setup 中的兜底）。
            // 若把可见性也交给插件恢复，两者会互相干扰。
            //
            // 插件在窗口就绪时自动恢复状态（on_window_ready），因此恢复发生在
            // 窗口显示之前，不会出现先闪一下默认尺寸再跳变的情况。
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
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
            commands::import_image,
            commands::exit_app,
        ])
        .setup(move |app| {
            menu::build(app.handle())?;

            let state = app.state::<AppState>();
            *state.startup_files.lock().unwrap() = startup_files.clone();

            // 首次启动时按屏幕可用区域收缩窗口。
            //
            // 配置里的 900×620 在小屏（如 1366×768）上会占掉八成高度，观感上
            // 「一打开就拉得特别高」。这里按工作区的比例上限收敛一次。
            // 若窗口状态插件已恢复过用户尺寸，则跳过——尊重用户的选择。
            clamp_window_to_screen(app.handle());

            // 主窗口初始为隐藏（见 tauri.conf.json 的 visible: false），由前端在
            // 首屏渲染完成后调用 show() 显示，以避免暴露 WebView 初始化期间的白屏。
            //
            // 兜底：若前端因脚本错误或权限问题未能显示窗口，用户将看不到任何界面。
            // 这里启动一个延时任务，超时后若窗口仍不可见就强制显示，保证应用始终可用。
            //
            // 窗口操作必须在主线程执行，因此通过 run_on_main_thread 派发，
            // 而不是直接在子线程里调用 show()。
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(3000));
                let h = handle.clone();
                let _ = handle.run_on_main_thread(move || {
                    if let Some(window) = h.get_webview_window("main") {
                        if !window.is_visible().unwrap_or(true) {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });
            });

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

/// 首次启动时把窗口尺寸收敛到屏幕可用区域内。
///
/// 仅在**没有历史尺寸记录**时执行。用户调整过窗口后，尺寸由 window-state 插件
/// 恢复，这里必须让位——否则会把用户的选择覆盖掉。
///
/// 判断依据是插件的状态文件是否存在。插件把它存在 app 配置目录下，
/// 文件名与插件默认值一致。
fn clamp_window_to_screen(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if has_saved_window_state(app) {
        return;
    }

    let Some(monitor) = window.current_monitor().ok().flatten() else {
        return;
    };

    let scale = monitor.scale_factor();
    let work = monitor.work_area();

    // 工作区逻辑尺寸（排除任务栏）
    let avail_w = work.size.width as f64 / scale;
    let avail_h = work.size.height as f64 / scale;

    let Ok(cur) = window.inner_size() else { return };
    let cur_w = cur.width as f64 / scale;
    let cur_h = cur.height as f64 / scale;

    // 上限：宽不超过工作区的 80%，高不超过 78%。
    // 取值偏保守——窗口顶到屏幕边缘会显得拥挤，也让桌面失去呼吸感。
    let max_w = avail_w * 0.80;
    let max_h = avail_h * 0.78;

    let target_w = cur_w.min(max_w).max(480.0);
    let target_h = cur_h.min(max_h).max(320.0);

    if (target_w - cur_w).abs() > 1.0 || (target_h - cur_h).abs() > 1.0 {
        let _ = window.set_size(tauri::LogicalSize::new(target_w, target_h));
        let _ = window.center();
    }
}

/// 判断 window-state 插件是否已有保存的窗口状态。
///
/// 文件名取自插件的公开常量而非硬编码字符串——插件若改名，这里会跟着变，
/// 不会静默失效。只做存在性检查，不解析内容。
fn has_saved_window_state(app: &tauri::AppHandle) -> bool {
    use tauri::Manager;
    let Ok(dir) = app.path().app_config_dir() else {
        return false;
    };
    dir.join(tauri_plugin_window_state::DEFAULT_FILENAME).exists()
}
