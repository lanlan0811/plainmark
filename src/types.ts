/** 与 Rust 侧 serde 序列化格式保持一致的类型定义。 */

/** 文件编码。与 Win10 记事本支持的集合一致。 */
export type Encoding =
  | "Ansi"
  | "Utf8"
  | "Utf8Bom"
  | "Utf16Le"
  | "Utf16Be";

/** 换行符风格。 */
export type Eol = "Crlf" | "Lf" | "Cr";

/** 打开或保存后的文档元信息。 */
export interface DocumentInfo {
  /** 无路径表示尚未保存的新文档 */
  path: string | null;
  /** 文件名，用于标签页与标题栏 */
  name: string;
  encoding: Encoding;
  eol: Eol;
  /** 文件内容，换行符已归一化为 \n */
  text: string;
  /** 最后修改时间（毫秒时间戳），用于检测外部改动；新文档为 null */
  modifiedMs: number | null;
}

/** 写盘结果。若因编码无法表示某些字符而失败，lossy 为 true。 */
export interface WriteResult {
  ok: boolean;
  /** 编码无法表示全部字符，需要用户确认是否放弃这些字符 */
  lossy: boolean;
  /** 无法表示的字符样本，用于提示用户 */
  lossySample: string[];
  modifiedMs: number | null;
}

/** 保存时的编码选择。null 表示沿用文档当前编码。 */
export interface SaveOptions {
  path: string;
  text: string;
  encoding: Encoding;
  eol: Eol;
}

/** 后端菜单事件。id 与 menu.rs 中注册的 id 一一对应。 */
export interface MenuEvent {
  id: string;
}

export const ENCODING_LABELS: Record<Encoding, string> = {
  Ansi: "ANSI",
  Utf8: "UTF-8",
  Utf8Bom: "UTF-8 BOM",
  Utf16Le: "UTF-16 LE",
  Utf16Be: "UTF-16 BE",
};

export const EOL_LABELS: Record<Eol, string> = {
  Crlf: "Windows (CRLF)",
  Lf: "Unix (LF)",
  Cr: "Macintosh (CR)",
};

/** 「另存为」对话框里可选的编码，顺序对齐记事本。 */
export const SAVE_ENCODINGS: Encoding[] = [
  "Ansi",
  "Utf16Le",
  "Utf16Be",
  "Utf8",
  "Utf8Bom",
];

/** 状态栏换行符下拉的顺序，对齐 Win11 记事本。 */
export const EOL_CHOICES: Eol[] = ["Crlf", "Lf", "Cr"];

/** 字号范围与步进，对齐记事本的缩放行为（10%–500%，每次 10%）。 */
export const ZOOM_MIN = 10;
export const ZOOM_MAX = 500;
export const ZOOM_STEP = 10;
