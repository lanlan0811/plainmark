//! 换行符检测与归一化。
//!
//! 内部一律用 `\n` 表示换行（CodeMirror 的要求），只在读写磁盘时转换。
//! 检测规则对齐记事本：以文件中**首个**出现的换行符为准。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eol {
    Crlf,
    Lf,
    Cr,
}

impl Eol {
    /// 状态栏显示名，与记事本逐字一致。
    pub fn label(self) -> &'static str {
        match self {
            Eol::Crlf => "Windows (CRLF)",
            Eol::Lf => "Unix (LF)",
            Eol::Cr => "Macintosh (CR)",
        }
    }

    /// 该换行符的实际字符。
    pub fn as_str(self) -> &'static str {
        match self {
            Eol::Crlf => "\r\n",
            Eol::Lf => "\n",
            Eol::Cr => "\r",
        }
    }
}

/// 检测文本使用的换行符。
///
/// 找到第一个换行就返回，因此混合换行的文件按首个判定——这与记事本一致，
/// 也意味着这类文件在首次保存时会被统一成所选换行符。
pub fn detect(text: &str) -> Eol {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                // \r\n 记作 CRLF；孤立 \r 记作 CR
                return if bytes.get(i + 1) == Some(&b'\n') {
                    Eol::Crlf
                } else {
                    Eol::Cr
                };
            }
            b'\n' => return Eol::Lf,
            _ => i += 1,
        }
    }
    // 没有换行符时按 Windows 处理，与记事本新建文件一致
    Eol::Crlf
}

/// 把任意换行符统一成 `\n`，供编辑器内部使用。
pub fn normalize(text: &str) -> String {
    // 先把 \r\n 折叠成 \n，再把剩余的孤立 \r 换成 \n
    if !text.contains('\r') {
        return text.to_owned();
    }
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// 把内部的 `\n` 还原成指定换行符，供写盘使用。
pub fn denormalize(text: &str, eol: Eol) -> String {
    match eol {
        Eol::Lf => text.to_owned(),
        Eol::Crlf => text.replace('\n', "\r\n"),
        Eol::Cr => text.replace('\n', "\r"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_crlf() {
        assert_eq!(detect("a\r\nb"), Eol::Crlf);
    }

    #[test]
    fn detects_lf() {
        assert_eq!(detect("a\nb"), Eol::Lf);
    }

    #[test]
    fn detects_cr() {
        assert_eq!(detect("a\rb"), Eol::Cr);
    }

    #[test]
    fn defaults_to_crlf_when_no_newline() {
        assert_eq!(detect("abc"), Eol::Crlf);
    }

    #[test]
    fn uses_first_occurrence_for_mixed() {
        // 首个是 LF，尽管后面有 CRLF
        assert_eq!(detect("a\nb\r\nc"), Eol::Lf);
    }

    #[test]
    fn normalize_folds_all_styles() {
        assert_eq!(normalize("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }

    #[test]
    fn normalize_is_noop_without_cr() {
        assert_eq!(normalize("a\nb"), "a\nb");
    }

    #[test]
    fn denormalize_roundtrip_crlf() {
        let original = "a\r\nb\r\n";
        assert_eq!(denormalize(&normalize(original), Eol::Crlf), original);
    }

    #[test]
    fn denormalize_roundtrip_cr() {
        let original = "a\rb\r";
        assert_eq!(denormalize(&normalize(original), Eol::Cr), original);
    }
}
