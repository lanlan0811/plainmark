//! 应用级共享状态。
//!
//! 这里刻意**不**对运行时泛型：plainmark 是桌面端应用，固定使用默认的 Wry
//! 运行时。用具体类型可以让 `#[tauri::command]` 里的 `State<'_, AppState>`
//! 直接解析，避免泛型参数无法推断的编译错误。

use std::sync::Mutex;

use tauri::menu::CheckMenuItem;

/// 应用状态。菜单勾选项的句柄需要保留，才能在运行期改勾选状态。
pub struct AppState {
    /// 格式 → 自动换行
    pub word_wrap_item: Mutex<Option<CheckMenuItem<tauri::Wry>>>,
    /// 查看 → 状态栏
    pub status_bar_item: Mutex<Option<CheckMenuItem<tauri::Wry>>>,
    /// 启动时通过命令行传入的文件路径，由前端取走后清空
    pub startup_files: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            word_wrap_item: Mutex::new(None),
            status_bar_item: Mutex::new(None),
            startup_files: Mutex::new(Vec::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
