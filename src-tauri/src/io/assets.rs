//! 图片等素材的导入。
//!
//! 拖入的图片会**复制**到文档同目录下的 `assets/` 中，Markdown 里插入相对路径。
//! 这样文档连同素材可以整体拷走而不丢图，也不会因为源文件被移动而失效。

use std::path::{Path, PathBuf};

/// 支持的图片扩展名（小写，不含点）。
const IMAGE_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "avif", "ico",
];

/// 素材目录名。
const ASSETS_DIR: &str = "assets";

/// 导入结果。
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedAsset {
    /// 可直接写进 Markdown 的相对路径，如 `assets/图片.png`
    pub relative_path: String,
    /// 落盘后的完整路径
    pub absolute_path: String,
    /// 原始文件名，用于生成 alt 文本
    pub name: String,
}

/// 判断扩展名是否为受支持的图片。
pub fn is_image(path: &Path) -> bool {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .map(|e| IMAGE_EXTS.contains(&e.as_str()))
        .unwrap_or(false)
}

/// 在目标目录里找一个不冲突的文件名。
///
/// 重名时按 `名字-1.png`、`名字-2.png` 递增，而不是直接覆盖——
/// 覆盖会静默毁掉用户已有的素材。
fn unique_path(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    // 拆出主名与扩展名，逐个尝试带序号的变体
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_string());
    let ext = path
        .extension()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    for n in 1..=9999 {
        let name = if ext.is_empty() {
            format!("{stem}-{n}")
        } else {
            format!("{stem}-{n}.{ext}")
        };
        let candidate = dir.join(&name);
        if !candidate.exists() {
            return candidate;
        }
    }

    // 极端情况下退回时间戳无关的兜底名
    dir.join(format!("{stem}-{}.{}", std::process::id(), ext))
}

/// 把源文件复制到 `文档目录/assets/` 下，返回相对路径与完整路径。
///
/// `doc_path` 是当前文档的完整路径；文档尚未保存时调用方应拒绝导入
/// （没有目录可放素材）。
pub fn import_image(doc_path: &Path, source: &Path) -> Result<ImportedAsset, String> {
    if !source.is_file() {
        return Err(format!("源文件不存在：{}", source.display()));
    }

    if !is_image(source) {
        return Err("只支持图片文件（png/jpg/gif/webp/bmp/svg 等）".to_string());
    }

    let doc_dir = doc_path
        .parent()
        .ok_or_else(|| "无法确定文档所在目录".to_string())?;

    let assets_dir = doc_dir.join(ASSETS_DIR);
    std::fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("无法创建素材目录：{e}"))?;

    let file_name = source
        .file_name()
        .ok_or_else(|| "源文件没有文件名".to_string())?
        .to_string_lossy()
        .into_owned();

    let dest = unique_path(&assets_dir, &file_name);

    std::fs::copy(source, &dest).map_err(|e| format!("复制图片失败：{e}"))?;

    let dest_name = dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or(file_name);

    // Markdown 里一律用正斜杠，保证在其它平台也有效
    let relative_path = format!("{ASSETS_DIR}/{dest_name}");

    Ok(ImportedAsset {
        relative_path,
        absolute_path: dest.to_string_lossy().into_owned(),
        name: dest_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("plainmark_assets_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn recognizes_image_extensions() {
        assert!(is_image(Path::new("a.png")));
        assert!(is_image(Path::new("a.JPG")));
        assert!(is_image(Path::new("a.svg")));
        assert!(!is_image(Path::new("a.txt")));
        assert!(!is_image(Path::new("a")));
    }

    #[test]
    fn copies_into_assets_dir_with_relative_path() {
        let dir = temp_dir("copy");
        let src = dir.join("图片.png");
        std::fs::write(&src, b"fake-png").unwrap();

        let doc = dir.join("note.md");
        let out = import_image(&doc, &src).unwrap();

        assert_eq!(out.relative_path, "assets/图片.png");
        assert!(Path::new(&out.absolute_path).is_file());
        assert!(dir.join("assets").is_dir());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn does_not_overwrite_existing_file() {
        let dir = temp_dir("dup");
        let src = dir.join("a.png");
        std::fs::write(&src, b"first").unwrap();

        let doc = dir.join("note.md");
        let first = import_image(&doc, &src).unwrap();
        let second = import_image(&doc, &src).unwrap();

        // 第二次应改名而不是覆盖
        assert_ne!(first.relative_path, second.relative_path);
        assert_eq!(second.relative_path, "assets/a-1.png");

        // 原文件内容必须还在
        let content = std::fs::read(&first.absolute_path).unwrap();
        assert_eq!(content, b"first");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_non_image() {
        let dir = temp_dir("reject");
        let src = dir.join("a.txt");
        std::fs::write(&src, b"text").unwrap();

        let doc = dir.join("note.md");
        assert!(import_image(&doc, &src).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
