//! 前端可调用的命令。所有文件系统操作都收口在这里。
//!
//! 窗口类命令统一用具体的 `tauri::Window`（即默认的 Wry 运行时），不做运行时
//! 泛型——这样 `#[tauri::command]` 的参数解析最直接，也最不容易出推断问题。

use std::path::Path;

use tauri::{AppHandle, State, Window};

use crate::io::{self, DocumentInfo, SaveOptions, WriteResult};
use crate::state::AppState;

/// 打开文件并解析编码与换行符。
#[tauri::command]
pub fn read_document(path: String) -> Result<DocumentInfo, String> {
    io::read(Path::new(&path))
}

/// 写回文件。
///
/// `force` 为 false 时，若编码无法表示全部字符则不落盘，返回 `lossy: true`
/// 交由前端弹窗确认；用户确认后前端再次调用并置 `force` 为 true。
#[tauri::command]
pub fn write_document(options: SaveOptions, force: bool) -> Result<WriteResult, String> {
    io::write(&options, force)
}

/// 取走启动时命令行传入的文件路径。取走后清空，避免重复打开。
#[tauri::command]
pub fn get_startup_files(state: State<'_, AppState>) -> Vec<String> {
    let mut files = state.startup_files.lock().unwrap();
    std::mem::take(&mut *files)
}

/// 记录当前窗口是否有未保存改动。
///
/// 关闭窗口前的拦截由前端 `onCloseRequested` 负责——只有前端知道每个标签页的
/// 脏状态。这个命令目前只用于状态跟踪，保留以便后续在 Rust 侧做关闭拦截。
#[tauri::command]
pub fn set_window_dirty(dirty: bool) -> Result<(), String> {
    let _ = dirty;
    Ok(())
}

/// 在指定窗口上设置标题。
#[tauri::command]
pub fn set_title(window: Window, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())
}

/// 关闭当前窗口。
#[tauri::command]
pub fn close_window(window: Window) -> Result<(), String> {
    window.destroy().map_err(|e| e.to_string())
}

/// 应用退出。
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// 判断路径是否已存在，用于「另存为」时的覆盖确认。
#[tauri::command]
pub fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}

/// 打开帮助页面（项目主页）。
#[tauri::command]
pub fn open_help() -> Result<(), String> {
    open_url("https://github.com/lanlan0811/plainmark");
    Ok(())
}

/// 用系统默认浏览器打开链接。
#[cfg(windows)]
fn open_url(url: &str) {
    use std::process::Command;
    // 交给 cmd 的 start 处理；第一个空参数是 start 的「窗口标题」占位，
    // 缺少它时带引号的 URL 会被误当成标题。
    let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
}

#[cfg(not(windows))]
fn open_url(url: &str) {
    use std::process::Command;
    let _ = Command::new("xdg-open").arg(url).spawn();
}
