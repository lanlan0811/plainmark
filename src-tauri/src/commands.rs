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

/// 记录前端上报的启动里程碑，输出到 stderr。
///
/// 用于定位启动耗时分布：Rust 侧无法感知 WebView 内部的加载进度，
/// 只能由前端在关键节点主动上报。日志仅在开启 `PLAINMARK_STARTUP_LOG`
/// 环境变量时输出，平时不产生噪音。
#[tauri::command]
pub fn report_startup_milestone(name: String) {
    if std::env::var_os("PLAINMARK_STARTUP_LOG").is_none() {
        return;
    }
    eprintln!(
        "[startup] {:>6} ms  {}",
        crate::ms_since_start(),
        name
    );
}

/// 关闭当前窗口。
#[tauri::command]
pub fn close_window(window: Window) -> Result<(), String> {
    window.destroy().map_err(|e| e.to_string())
}

/// 退出整个应用。
///
/// 与 `close_window` 的区别：后者只关当前窗口，多窗口时其余窗口仍在。
/// 「文件 → 退出」应该关掉整个应用，与记事本一致。
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// 取文件的最后修改时间，用于检测外部改动。
///
/// 返回 None 表示文件不存在或无法访问——调用方据此判断文件已被删除/移动。
#[tauri::command]
pub fn file_modified_ms(path: String) -> Option<u64> {
    io::modified_ms(Path::new(&path))
}

/// 把图片复制到文档同目录的 `assets/` 下，返回可写进 Markdown 的相对路径。
///
/// 文档尚未保存（没有路径）时无法确定素材放哪，由前端先走「另存为」。
#[tauri::command]
pub fn import_image(doc_path: String, source_path: String) -> Result<io::ImportedAsset, String> {
    io::import_image(Path::new(&doc_path), Path::new(&source_path))
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
