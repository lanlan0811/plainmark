/**
 * Markdown → HTML 渲染。
 *
 * 三个关键设计：
 *
 * 1. **安全性**：文档可能来自任何地方，渲染结果一律经 DOMPurify 清洗。
 *    markdown-it 默认允许内联 HTML，不清洗就等于给自己开 XSS 口子。
 *
 * 2. **成组渲染**：markdown-it 的顶层块是成对的 token（如 `bullet_list_open`
 *    与 `bullet_list_close`）。单独渲染 `_open` 会产生未闭合的 `<ul>`，
 *    因此必须按「顶层块」为单位整组渲染。
 *
 * 3. **源码位置映射**：拖拽重排需要知道每个预览块对应源码的哪一段。
 *    给每个块包一层带 `data-src-start` / `data-src-end` 的容器，值是源码字符
 *    偏移量（markdown-it 的 token.map 给的是行号，需换算）。重排时据此改写源码，
 *    而不是靠文本内容去猜测匹配。
 */

import MarkdownIt from "markdown-it";
import DOMPurify from "dompurify";

/** 渲染结果。 */
export interface RenderedPreview {
  html: string;
}

/** 可重排的源码区间。 */
export interface SourceBlock {
  start: number;
  end: number;
}

const md = new MarkdownIt({
  html: true, // 允许内联 HTML（如 <br>、<img>），随后由 DOMPurify 清洗
  linkify: true, // 裸 URL 自动转链接
  breaks: false, // 单个换行不强制断行，与 GitHub 渲染一致
  typographer: false, // 不做引号/破折号智能替换，保持源码原样
});

/** 算出源码中每一行的起始字符偏移。 */
function lineOffsets(src: string): number[] {
  const offsets = [0];
  for (let i = 0; i < src.length; i++) {
    if (src.charCodeAt(i) === 10 /* \n */) offsets.push(i + 1);
  }
  return offsets;
}

/**
 * 把 token 流切成「顶层块」分组。
 *
 * markdown-it 只在块的**起始** token 上带 map，结束 token（如 `heading_close`）
 * 没有 map。因此以「level 为 0 且带 map」为分组边界，就能得到完整的成对区间。
 */
type TokenList = ReturnType<typeof md.parse>;

function groupTopLevelTokens(
  tokens: TokenList,
): { from: number; to: number; startLine: number; endLine: number }[] {
  const groups: { from: number; to: number; startLine: number; endLine: number }[] = [];

  let i = 0;
  while (i < tokens.length) {
    const token = tokens[i]!;
    if (token.level === 0 && token.map) {
      // 找到下一个顶层块起点，中间的都是本组的成员
      let j = i + 1;
      while (j < tokens.length) {
        const next = tokens[j]!;
        if (next.level === 0 && next.map) break;
        j++;
      }
      groups.push({
        from: i,
        to: j,
        startLine: token.map[0]!,
        endLine: token.map[1]!,
      });
      i = j;
    } else {
      // 理论上不该出现（顶层 token 都有 map），跳过以免死循环
      i++;
    }
  }

  return groups;
}

/**
 * 渲染 Markdown 为安全的 HTML。
 *
 * 每个顶层块会被包进 `<div class="pm-block" data-src-start data-src-end>`，
 * 供拖拽重排定位源码。
 */
export function renderMarkdown(src: string): RenderedPreview {
  if (!src.trim()) return { html: "" };

  const env: Record<string, unknown> = {};
  const tokens = md.parse(src, env);
  const offsets = lineOffsets(src);
  const groups = groupTopLevelTokens(tokens);

  const parts: string[] = [];
  for (const g of groups) {
    // 整组渲染，保证 open/close 配对
    const inner = md.renderer.render(tokens.slice(g.from, g.to), md.options, env);
    const start = offsets[g.startLine] ?? 0;
    // 结束行是开区间，offsets[endLine] 即该块之后的第一个字符
    const end = offsets[g.endLine] ?? src.length;

    parts.push(
      `<div class="pm-block" data-src-start="${start}" data-src-end="${end}">${inner}</div>`,
    );
  }

  const raw = parts.join("\n");

  const clean = DOMPurify.sanitize(raw, {
    ADD_ATTR: ["data-src-start", "data-src-end"],
    // 只放行常见安全协议，挡掉 javascript: 之类
    ALLOWED_URI_REGEXP: /^(?:https?|mailto|tel|file|data):|^[^a-z]|^[a-z+.-]+(?:[^a-z+.:-]|$)/i,
  });

  return { html: clean };
}

/**
 * 从渲染好的容器里取出所有可重排块的源码区间。
 * 拖拽重排据此定位要移动的源码片段。
 */
export function extractBlocks(container: HTMLElement): SourceBlock[] {
  const nodes = container.querySelectorAll<HTMLElement>(".pm-block[data-src-start]");
  const blocks: SourceBlock[] = [];

  nodes.forEach((el) => {
    const start = Number(el.dataset.srcStart);
    const end = Number(el.dataset.srcEnd);
    if (Number.isFinite(start) && Number.isFinite(end) && end > start) {
      blocks.push({ start, end });
    }
  });

  return blocks;
}

/**
 * 按新的顺序重组源码。
 *
 * 为什么不直接对调两段切片：markdown-it 给的块区间对尾随空行的归属并不一致
 * ——列表块的 map 会包含其后的空行，引用块则不含。直接拼接会让相邻两块之间
 * 只剩单个换行，Markdown 会把它们合并成同一段落（`"CCC\nBBB"` 变成一段）。
 *
 * 因此改为：取出每块的正文（去掉尾部换行），按新顺序用空行连接。
 * 副作用是块之间的空行会被规范成恰好一个——这是 Markdown 的通行写法，
 * 且只在真正发生重排时才重写文件。
 *
 * @param src    原始源码
 * @param blocks 各块的源码区间，需按文档顺序排列
 * @param order  新的顺序，元素是 blocks 的下标
 */
export function reorderBlocks(
  src: string,
  blocks: SourceBlock[],
  order: number[],
): string {
  if (blocks.length === 0) return src;

  // 文档首尾的空白原样保留，只重排块之间的部分
  const head = src.slice(0, blocks[0]!.start);
  const lastBlock = blocks[blocks.length - 1]!;
  const tail = src.slice(lastBlock.end);

  const pieces = order.map((i) => {
    const b = blocks[i]!;
    // 去掉尾部换行，统一由 join 补空行
    return src.slice(b.start, b.end).replace(/\n+$/, "");
  });

  return head + pieces.join("\n\n") + "\n" + tail.replace(/^\n+/, "");
}

/**
 * 把某个块移动到目标位置（拖拽重排的语义）。
 *
 * @param fromIndex 被拖动块的下标
 * @param toIndex   放置目标块的下标（被拖动块会占据这个位置）
 */
export function moveBlockTo(
  src: string,
  blocks: SourceBlock[],
  fromIndex: number,
  toIndex: number,
): string {
  if (fromIndex === toIndex) return src;
  if (fromIndex < 0 || fromIndex >= blocks.length) return src;
  if (toIndex < 0 || toIndex >= blocks.length) return src;

  const order = blocks.map((_, i) => i);
  const [moved] = order.splice(fromIndex, 1);
  order.splice(toIndex, 0, moved!);

  return reorderBlocks(src, blocks, order);
}
