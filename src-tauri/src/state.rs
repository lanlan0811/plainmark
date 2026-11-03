//! 应用级共享状态。
//!
//! 这里刻意**不**对运行时泛型：plainmark 是桌面端应用，固定使用默认的 Wry
//! 运行时。用具体类型可以让 `#[tauri::command]` 里的 `State<'_, AppState>`
//! 直接解析，避免泛型参数无法推断的编译错误。

use std::sync::Mutex;

/// 应用状态。
pub struct AppState {
    /// 启动时通过命令行传入的文件路径，由前端取走后清空
    pub startup_files: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            startup_files: Mutex::new(Vec::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
