//! 文件读写：编码检测、换行符处理与落盘。

pub mod document;
pub mod encoding;
pub mod eol;

pub use document::{modified_ms, read, write, DocumentInfo, SaveOptions, WriteResult};
pub use encoding::TextEncoding;
pub use eol::Eol;
