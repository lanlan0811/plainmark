//! 编码检测与编解码。
//!
//! 本模块刻意**不使用**统计式编码探测（如 chardetng）。记事本的判断逻辑很简单：
//! 先看 BOM，再看能否按严格 UTF-8 解码，否则回退到系统 ANSI 代码页。只有复刻这条
//! 确定性链路，plainmark 的判定结果才会和记事本一致。
//!
//! 已知的固有歧义（与记事本相同，不是缺陷）：
//! - 真正的 GBK 文件若字节恰好构成合法 UTF-8，会被判为 UTF-8；
//! - 纯 ASCII 的无 BOM 文件与 ANSI 无法区分，但两者解码结果相同，无可见差异。

use encoding_rs::{Encoding, BIG5, EUC_KR, GBK, SHIFT_JIS, UTF_16BE, UTF_16LE, UTF_8, WINDOWS_1252};
use serde::{Deserialize, Serialize};

/// 支持的编码集合，与 Win10 记事本一致（不支持 UTF-32）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextEncoding {
    /// 系统 ANSI 代码页（简体中文系统上通常是 GBK）
    Ansi,
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
}

impl TextEncoding {
    /// 状态栏显示名。UTF-8 BOM 单独标注，便于和记事本核对。
    pub fn label(self) -> &'static str {
        match self {
            TextEncoding::Ansi => "ANSI",
            TextEncoding::Utf8 => "UTF-8",
            TextEncoding::Utf8Bom => "UTF-8 BOM",
            TextEncoding::Utf16Le => "UTF-16 LE",
            TextEncoding::Utf16Be => "UTF-16 BE",
        }
    }
}

/// 解码结果。
pub struct Decoded {
    pub text: String,
    pub encoding: TextEncoding,
}

/// 把系统 ANSI 代码页号映射到 encoding_rs 的编码。
///
/// 覆盖常见的东亚与西欧代码页。未收录的代码页回退到 WINDOWS_1252，
/// 这与记事本在无法识别时的行为接近。
fn ansi_encoding() -> &'static Encoding {
    match system_codepage() {
        936 => GBK,            // 简体中文
        950 => BIG5,           // 繁体中文
        932 => SHIFT_JIS,      // 日文
        949 => EUC_KR,         // 韩文
        _ => WINDOWS_1252,     // 西欧及其它
    }
}

/// 读取系统 ANSI 代码页（Win32 GetACP）。
///
/// 直接声明 FFI 而不引入 `windows` crate：只需要这一个函数，省掉一个依赖
/// 和它的 feature 配置。
#[cfg(windows)]
fn system_codepage() -> u32 {
    extern "system" {
        fn GetACP() -> u32;
    }
    // SAFETY: GetACP 无参数、无副作用，返回当前进程的 ANSI 代码页，总是成功。
    unsafe { GetACP() }
}

/// 非 Windows 平台（仅用于开发机上的单元测试）固定按 UTF-8 处理。
#[cfg(not(windows))]
fn system_codepage() -> u32 {
    65001
}

/// 按 BOM → 严格 UTF-8 → ANSI 的顺序检测并解码。
pub fn decode(bytes: &[u8]) -> Decoded {
    // 1. BOM 优先，这是最可靠的信号
    if let Some((enc, _bom_len)) = Encoding::for_bom(bytes) {
        // for_bom 会识别 UTF-8 / UTF-16LE / UTF-16BE 的 BOM
        let (text, encoding) = if enc == UTF_8 {
            (UTF_8.decode_with_bom_removal(bytes).0, TextEncoding::Utf8Bom)
        } else if enc == UTF_16LE {
            (UTF_16LE.decode_with_bom_removal(bytes).0, TextEncoding::Utf16Le)
        } else {
            (UTF_16BE.decode_with_bom_removal(bytes).0, TextEncoding::Utf16Be)
        };
        return Decoded {
            text: text.into_owned(),
            encoding,
        };
    }

    // 2. 无 BOM：严格的 UTF-8 校验。用 std 的校验器，它比 encoding_rs 更严格。
    if let Ok(s) = std::str::from_utf8(bytes) {
        // 含 NUL 的文本按 ANSI 处理，避免把二进制误判成文本
        if !s.contains('\0') {
            return Decoded {
                text: s.to_owned(),
                encoding: TextEncoding::Utf8,
            };
        }
    }

    // 3. 回退到 ANSI。此处的替换字符与记事本一致，不做额外补救。
    let (text, _, _) = ansi_encoding().decode(bytes);
    Decoded {
        text: text.into_owned(),
        encoding: TextEncoding::Ansi,
    }
}

/// 编码结果。`unmappable` 列出无法表示的字符样本（最多 10 个）。
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub unmappable: Vec<String>,
}

/// 按指定编码把文本编成字节。换行符由调用方在文本里准备好。
///
/// 关于 BOM：`encoding_rs` 的 `Encoding::encode()` **从不写 BOM**，而且对纯 ASCII
/// 输入有直通优化——对 UTF-16 目标也会直接返回 UTF-8 字节。所以 UTF-8 与 UTF-16
/// 这两种情况一律手写，只有 ANSI 走 `encode()`。
pub fn encode(text: &str, encoding: TextEncoding) -> Encoded {
    let bytes = match encoding {
        // 无 BOM 的 UTF-8 就是原始字节
        TextEncoding::Utf8 => text.as_bytes().to_vec(),

        // 有 BOM 的 UTF-8：EF BB BF + 原始字节
        TextEncoding::Utf8Bom => {
            let mut out = Vec::with_capacity(text.len() + 3);
            out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            out.extend_from_slice(text.as_bytes());
            out
        }

        TextEncoding::Utf16Le => encode_utf16(text, true),
        TextEncoding::Utf16Be => encode_utf16(text, false),

        TextEncoding::Ansi => {
            let enc = ansi_encoding();
            let (cow, _, had_unmappable) = enc.encode(text);
            let unmappable = if had_unmappable {
                collect_unmappable(text, enc)
            } else {
                Vec::new()
            };
            return Encoded {
                bytes: cow.into_owned(),
                unmappable,
            };
        }
    };

    // UTF-8 与 UTF-16 能表示所有 Rust 字符，不存在有损情况
    Encoded {
        bytes,
        unmappable: Vec::new(),
    }
}

/// 手写 UTF-16 编码，并前置对应字节序的 BOM。
///
/// 不能复用 `encoding_rs`：它的 `encode()` 对纯 ASCII 输入会走直通分支，
/// 直接返回 UTF-8 字节，用于 UTF-16 目标时结果是错的。
fn encode_utf16(text: &str, little_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 2 + 2);
    out.extend_from_slice(if little_endian {
        &[0xFF, 0xFE]
    } else {
        &[0xFE, 0xFF]
    });
    for unit in text.encode_utf16() {
        out.extend_from_slice(&if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        });
    }
    out
}

/// 找出目标编码无法表示的字符，去重后最多返回 10 个。
fn collect_unmappable(text: &str, enc: &'static Encoding) -> Vec<String> {
    let mut seen = Vec::new();
    for ch in text.chars() {
        if seen.len() >= 10 {
            break;
        }
        let s = ch.to_string();
        let (_, _, unmappable) = enc.encode(&s);
        if unmappable && !seen.contains(&s) {
            seen.push(s);
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 编码再解码应还原原文，且编码结果被识别为预期编码。
    /// 这些测试一律走本模块自己的 encode/decode —— 若直接用 encoding_rs，
    /// 就发现不了「UTF-16 目标被 ASCII 直通写成 UTF-8」这类缺陷。
    fn roundtrip(text: &str, enc: TextEncoding) {
        let encoded = encode(text, enc);
        let decoded = decode(&encoded.bytes);
        assert_eq!(
            decoded.text, text,
            "{enc:?} 往返后文本不一致（字节：{:02X?}）",
            encoded.bytes
        );
        assert_eq!(decoded.encoding, enc, "{enc:?} 往返后编码识别不一致");
    }

    #[test]
    fn utf8_roundtrip_no_bom() {
        roundtrip("hello 中文", TextEncoding::Utf8);
        let e = encode("abc", TextEncoding::Utf8);
        assert_eq!(e.bytes, b"abc", "UTF-8 无 BOM 不应有前缀");
    }

    #[test]
    fn utf8_bom_roundtrip() {
        roundtrip("abc", TextEncoding::Utf8Bom);
        roundtrip("标题", TextEncoding::Utf8Bom);
        let e = encode("abc", TextEncoding::Utf8Bom);
        assert_eq!(&e.bytes[..3], &[0xEF, 0xBB, 0xBF], "必须带 UTF-8 BOM");
        assert_eq!(&e.bytes[3..], b"abc");
    }

    #[test]
    fn utf16le_roundtrip() {
        // 关键回归：纯 ASCII 输入也必须编成 UTF-16，而不是被直通成 UTF-8
        roundtrip("abc", TextEncoding::Utf16Le);
        roundtrip("你好世界", TextEncoding::Utf16Le);
        let e = encode("abc", TextEncoding::Utf16Le);
        assert_eq!(
            e.bytes,
            vec![0xFF, 0xFE, 0x61, 0x00, 0x62, 0x00, 0x63, 0x00],
            "ASCII 输入也必须是 UTF-16LE 字节"
        );
    }

    #[test]
    fn utf16be_roundtrip() {
        roundtrip("abc", TextEncoding::Utf16Be);
        roundtrip("你好世界", TextEncoding::Utf16Be);
        let e = encode("abc", TextEncoding::Utf16Be);
        assert_eq!(
            e.bytes,
            vec![0xFE, 0xFF, 0x00, 0x61, 0x00, 0x62, 0x00, 0x63],
            "ASCII 输入也必须是 UTF-16BE 字节"
        );
    }

    #[test]
    fn ansi_roundtrip_ascii() {
        roundtrip("plain ascii", TextEncoding::Ansi);
    }

    #[test]
    fn detects_utf8_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("标题".as_bytes());
        let d = decode(&bytes);
        assert_eq!(d.encoding, TextEncoding::Utf8Bom);
        assert_eq!(d.text, "标题");
    }

    #[test]
    fn detects_utf16le_bom() {
        let d = decode(&[0xFF, 0xFE, 0x60, 0x4F, 0x7D, 0x59]);
        assert_eq!(d.encoding, TextEncoding::Utf16Le);
        assert_eq!(d.text, "你好");
    }

    #[test]
    fn detects_utf16be_bom() {
        let d = decode(&[0xFE, 0xFF, 0x4F, 0x60, 0x59, 0x7D]);
        assert_eq!(d.encoding, TextEncoding::Utf16Be);
        assert_eq!(d.text, "你好");
    }

    #[test]
    fn plain_ascii_is_utf8_without_bom() {
        let d = decode(b"hello\nworld");
        assert_eq!(d.encoding, TextEncoding::Utf8);
        assert_eq!(d.text, "hello\nworld");
    }

    #[test]
    fn utf8_without_bom_is_detected() {
        let d = decode("中文内容".as_bytes());
        assert_eq!(d.encoding, TextEncoding::Utf8);
        assert_eq!(d.text, "中文内容");
    }

    #[test]
    fn nul_bytes_fall_back_to_ansi() {
        // 含 NUL 的内容不按 UTF-8 处理，避免把二进制误判成文本
        let d = decode(b"ab\0cd");
        assert_eq!(d.encoding, TextEncoding::Ansi);
    }

    #[test]
    fn ansi_flags_unmappable_chars() {
        // emoji 无法用 GBK 或 1252 表示
        let e = encode("hi 😀", TextEncoding::Ansi);
        assert!(!e.unmappable.is_empty());
    }

    #[test]
    fn ansi_ascii_is_clean() {
        let e = encode("plain ascii", TextEncoding::Ansi);
        assert!(e.unmappable.is_empty());
    }

    #[test]
    fn utf8_and_utf16_never_report_lossy() {
        for enc in [
            TextEncoding::Utf8,
            TextEncoding::Utf8Bom,
            TextEncoding::Utf16Le,
            TextEncoding::Utf16Be,
        ] {
            let e = encode("emoji 😀 也能表示", enc);
            assert!(e.unmappable.is_empty(), "{enc:?} 不应有无法表示的字符");
        }
    }
}
