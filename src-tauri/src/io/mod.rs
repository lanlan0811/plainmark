//! 文件读写：编码检测、换行符处理、素材导入与落盘。

pub mod assets;
pub mod document;
pub mod encoding;
pub mod eol;

pub use assets::{import_image, ImportedAsset};
pub use document::{modified_ms, read, write, DocumentInfo, SaveOptions, WriteResult};
pub use encoding::TextEncoding;
pub use eol::Eol;
