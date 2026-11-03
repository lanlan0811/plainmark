//! 文档的读盘与写盘。编码与换行符的细节都在这里收口。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::encoding::{self, TextEncoding};
use super::eol::{self, Eol};

/// 打开文件后返回给前端的完整信息。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInfo {
    pub path: String,
    pub name: String,
    /// 已归一化为 `\n`
    pub text: String,
    pub encoding: TextEncoding,
    pub eol: Eol,
    /// 文件最后修改时间（毫秒时间戳），用于外部改动检测
    pub modified_ms: Option<u64>,
}

/// 前端发来的保存请求。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOptions {
    pub path: String,
    /// 换行符已归一化为 `\n`
    pub text: String,
    pub encoding: TextEncoding,
    pub eol: Eol,
}

/// 写盘结果。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    pub ok: bool,
    /// 编码无法表示全部字符；调用方应先向用户确认再强制写入
    pub lossy: bool,
    /// 无法表示的字符样本，用于提示
    pub lossy_sample: Vec<String>,
    pub modified_ms: Option<u64>,
}

/// 从路径取文件名，用于标签页与标题栏。
fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// 读取文件的修改时间（毫秒时间戳）。
fn modified_ms_of(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
}

/// 只取修改时间，不读文件内容。用于外部改动检测——大文件下避免整份读取。
pub fn modified_ms(path: &Path) -> Option<u64> {
    modified_ms_of(path)
}

/// 打开文件：读字节 → 检测编码 → 检测换行符 → 归一化换行。
pub fn read(path: &Path) -> Result<DocumentInfo, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("无法读取文件：{e}"))?;

    let decoded = encoding::decode(&bytes);
    let eol = eol::detect(&decoded.text);
    let text = eol::normalize(&decoded.text);

    Ok(DocumentInfo {
        path: path.to_string_lossy().into_owned(),
        name: file_name_of(path),
        text,
        encoding: decoded.encoding,
        eol,
        modified_ms: modified_ms_of(path),
    })
}

/// 写盘：按指定编码与换行符编码 → 落盘。
///
/// ANSI 编码下若有字符无法表示，返回 `lossy: true` 且**不写盘**，
/// 交由前端弹窗向用户确认——这与记事本的行为一致。
pub fn write(options: &SaveOptions, force: bool) -> Result<WriteResult, String> {
    let path = PathBuf::from(&options.path);

    // 换行符从内部 \n 还原成目标风格
    let payload = eol::denormalize(&options.text, options.eol);
    let encoded = encoding::encode(&payload, options.encoding);

    if !encoded.unmappable.is_empty() && !force {
        // 未获用户确认，不落盘
        return Ok(WriteResult {
            ok: false,
            lossy: true,
            lossy_sample: encoded.unmappable,
            modified_ms: modified_ms_of(&path),
        });
    }

    std::fs::write(&path, &encoded.bytes).map_err(|e| format!("无法写入文件：{e}"))?;

    Ok(WriteResult {
        ok: true,
        lossy: !encoded.unmappable.is_empty(),
        lossy_sample: encoded.unmappable,
        modified_ms: modified_ms_of(&path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("plainmark_test_{name}"));
        p
    }

    #[test]
    fn read_write_roundtrip_utf8_crlf() {
        let path = temp_path("utf8_crlf.md");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all("标题\r\n正文".as_bytes()).unwrap();
        }

        let info = read(&path).unwrap();
        assert_eq!(info.text, "标题\n正文");
        assert_eq!(info.eol, Eol::Crlf);
        assert_eq!(info.encoding, TextEncoding::Utf8);

        let result = write(
            &SaveOptions {
                path: path.to_string_lossy().into_owned(),
                text: info.text.clone(),
                encoding: info.encoding,
                eol: info.eol,
            },
            false,
        )
        .unwrap();
        assert!(result.ok);

        // 字节应与原文件完全一致
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes, "标题\r\n正文".as_bytes());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_write_roundtrip_utf8_bom() {
        let path = temp_path("utf8_bom.md");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
            f.write_all("带BOM".as_bytes()).unwrap();
        }

        let info = read(&path).unwrap();
        assert_eq!(info.encoding, TextEncoding::Utf8Bom);
        assert_eq!(info.text, "带BOM");

        write(
            &SaveOptions {
                path: path.to_string_lossy().into_owned(),
                text: info.text.clone(),
                encoding: info.encoding,
                eol: info.eol,
            },
            false,
        )
        .unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF], "BOM 必须保留");
        assert_eq!(&bytes[3..], "带BOM".as_bytes());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ansi_lossy_write_is_refused_without_force() {
        let path = temp_path("ansi_lossy.txt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"ascii").unwrap();
        }

        let result = write(
            &SaveOptions {
                path: path.to_string_lossy().into_owned(),
                text: "含 emoji 😀 的文本".into(),
                encoding: TextEncoding::Ansi,
                eol: Eol::Crlf,
            },
            false,
        )
        .unwrap();

        assert!(!result.ok, "未确认时不应写盘");
        assert!(result.lossy);
        assert!(!result.lossy_sample.is_empty());

        // 原文件内容不应被改动
        assert_eq!(std::fs::read(&path).unwrap(), b"ascii");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ansi_lossy_write_succeeds_with_force() {
        let path = temp_path("ansi_force.txt");
        {
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(b"ascii").unwrap();
        }

        let result = write(
            &SaveOptions {
                path: path.to_string_lossy().into_owned(),
                text: "含 emoji 😀 的文本".into(),
                encoding: TextEncoding::Ansi,
                eol: Eol::Crlf,
            },
            true,
        )
        .unwrap();

        assert!(result.ok);
        assert!(result.lossy, "强制写入后仍应报告有损");

        let _ = std::fs::remove_file(&path);
    }
}
