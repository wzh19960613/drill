import katex from 'katex'

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

const NUL = '\u0000'

function katexHtml(tex: string, display: boolean): string {
  try {
    return katex.renderToString(tex, {
      displayMode: display,
      throwOnError: false,
      strict: 'ignore',
    })
  } catch {
    return `<code>${escapeHtml(tex)}</code>`
  }
}

export function richText(src: string): string {
  return richTextBase(src, false)
}

export function richTextSingle(src: string): string {
  return richTextBase(src, true)
}

interface Slotted {
  s: string
  maths: { tex: string; display: boolean }[]
  codes: string[]
}

/** Pull out $$…$$ / $…$ math and `inline code` first: their content must
 *  not be touched by the other inline rules. */
function extractSlots(src: string, single: boolean): Slotted {
  const maths: { tex: string; display: boolean }[] = []
  const codes: string[] = []
  let s = src.replace(/\$\$([\s\S]+?)\$\$/g, (_m, tex: string) => {
    maths.push({ tex, display: !single })
    return `${NUL}m${maths.length - 1}${NUL}`
  })
  s = s.replace(/\$([^$\n]+?)\$/g, (_m, tex: string) => {
    maths.push({ tex, display: false })
    return `${NUL}m${maths.length - 1}${NUL}`
  })
  s = s.replace(/`([^`\n]+?)`/g, (_m, code: string) => {
    codes.push(code)
    return `${NUL}c${codes.length - 1}${NUL}`
  })
  return { s, maths, codes }
}

function styleText(s: string): string {
  s = escapeHtml(s)
  s = s.replace(/\*\*([\s\S]+?)\*\*/g, '<strong>$1</strong>')
  s = s.replace(/\*([^*\n]+)\*/g, '<em>$1</em>')
  return s.replace(
    /\[([^\]\n]+)\]\((https?:\/\/[^\s)]+|\/[^\s)]*)\)/g,
    '<a href="$2" target="_blank" rel="noopener">$1</a>',
  )
}

/** Collapse the SINGLE-NEWLINE separator between adjacent DISPLAY-math
 *  placeholders: consecutive $$…$$ blocks in one paragraph belong to one
 *  visual group (the PDF merges them too). A blank line (\n\n) means the
 *  caller handed us unsplit paragraphs — that must stay two separate
 *  blocks, so only `[ \t]* \n [ \t]*` collapses, never `\s+`.
 *  Loop: a global replace consumes one side of each pair, re-run until
 *  stable. */
function collapseDisplayPairs(s: string, maths: { display: boolean }[]): string {
  const pair = new RegExp(`${NUL}m(\\d+)${NUL}[^\\S\\n]*\\n[^\\S\\n]*${NUL}m(\\d+)${NUL}`)
  for (let prev = ''; prev !== s; ) {
    prev = s
    s = s.replace(pair, (m, a: string, b: string) =>
      maths[Number(a)]?.display && maths[Number(b)]?.display
        ? `${NUL}m${a}${NUL}${NUL}m${b}${NUL}`
        : m,
    )
  }
  return s
}

/** Math stays a placeholder until here: rendering it earlier would expose
 *  KaTeX's HTML to the newline replacement and break the newlines inside
 *  its internal SVG paths. Adjacent display-math containers merge into one
 *  centered group (`class="q-math"` never occurs inside KaTeX output, so
 *  the boundary is unambiguous). */
function restoreSlots(s: string, maths: { tex: string; display: boolean }[], codes: string[]): string {
  s = s.replace(new RegExp(`${NUL}c(\\d+)${NUL}`, 'g'), (_m, i: string) => {
    const code = codes[Number(i)]
    return code === undefined ? '' : `<code class="md-icode">${escapeHtml(code)}</code>`
  })
  return s
    .replace(new RegExp(`${NUL}m(\\d+)${NUL}`, 'g'), (_m, i: string) => {
      const item = maths[Number(i)]
      if (!item) return ''
      return item.display
        ? `<div class="q-math">${katexHtml(item.tex, true)}</div>`
        : katexHtml(item.tex, false)
    })
    .replaceAll('</div><div class="q-math">', '<br class="q-math-sep"/>')
}

function richTextBase(src: string, single: boolean): string {
  const { s, maths, codes } = extractSlots(src, single)
  let out = styleText(s)
  out = collapseDisplayPairs(out, maths)
  out = out.replace(/\n/g, single ? ' ' : '<br/>')
  return restoreSlots(out, maths, codes)
}

/** LaTeX 数学内容的可见部分：去掉命令名与结构符，保留字母/数字/CJK */
function visibleMath(tex: string): string {
  return tex.replace(/\\[a-zA-Z]+/g, '').replace(/[{}^_]/g, '')
}

/** 答案的"显示长度"：数学按渲染后可见内容计，格式标记（加粗/行内代码/
 *  图片嵌入）不计，其余按字符数。用于限制居中胶囊只用于短答案。 */
export function displayLength(src: string): number {
  let s = src.replace(/!\[\[[^\]]*\]\]/g, '')
  s = s.replace(/\$\$([\s\S]+?)\$\$/g, (_m, tex: string) => visibleMath(tex))
  s = s.replace(/\$([^$\n]+?)\$/g, (_m, tex: string) => visibleMath(tex))
  s = s.replace(/\*\*/g, '').replace(/`([^`]*)`/g, '$1')
  return [...s].length
}

export interface BlockRenderOpts {
  /** Resolve a wiki-image name to a custom src (temp uploads); falls back
   *  to the assets route. */
  srcFor?: (name: string) => string | undefined
  /** Style `(A)` option blocks with the letter highlighted (题干 editor). */
  optionLetters?: boolean
}

const IMG_RE = /^!\[\[(.+?\.(?:svg|png|jpe?g|webp|gif))\]\]$/
const UL_RE = /^[-*+]\s+/
const OL_RE = /^\d+[.)、]\s+/
const CODE_FENCE_RE = /^```/

function splitRow(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((c) => c.trim())
}

function isSepRow(line: string): boolean {
  const t = line.trim()
  return (
    t.includes('-') &&
    t.includes('|') &&
    [...t].every((c) => '-:| \t'.includes(c))
  )
}

/** `| a | b |` + separator + rows (mirrors the backend) */
function tableHtml(block: string): string | null {
  const lines = block.split('\n').map((l) => l.trim())
  if (lines.length < 2 || !lines[0].includes('|') || !isSepRow(lines[1])) return null
  const head = splitRow(lines[0])
  const rows = lines.slice(2).map(splitRow)
  const th = head.map((c) => `<th>${richText(c)}</th>`).join('')
  const trs = rows
    .map((r) => `<tr>${r.map((c) => `<td>${richText(c)}</td>`).join('')}</tr>`)
    .join('')
  return `<table class="md-table"><thead><tr>${th}</tr></thead><tbody>${trs}</tbody></table>`
}

/** consecutive `- ` / `1. ` lines; non-list lines continue the previous item */
function listHtml(block: string): string | null {
  const lines = block.split('\n')
  const first = lines[0]?.trim() ?? ''
  const ordered = OL_RE.test(first)
  if (!ordered && !UL_RE.test(first)) return null
  const items: string[] = []
  for (const line of lines) {
    const t = line.trim()
    const m = ordered ? t.replace(OL_RE, '') : t.replace(UL_RE, '')
    if ((ordered && OL_RE.test(t)) || (!ordered && UL_RE.test(t))) {
      items.push(m)
    } else if (items.length) {
      items[items.length - 1] += `\n${t}`
    } else {
      return null
    }
  }
  const lis = items.map((i) => `<li>${richText(i)}</li>`).join('')
  return ordered ? `<ol class="md-list">${lis}</ol>` : `<ul class="md-list">${lis}</ul>`
}

export function paraHtml(p: string, opts: BlockRenderOpts = {}): string {
  const img = p.match(IMG_RE)
  if (img) {
    const name = img[1]
    const src = opts.srcFor?.(name) ?? `/api/assets/${encodeURIComponent(name)}`
    return `<div class="q-svg-wrap"><img class="q-svg" src="${src}" alt="${escapeHtml(name)}" loading="lazy" /></div>`
  }
  const heading = p.match(/^(#{1,6})\s+([\s\S]*)$/)
  if (heading) {
    const level = Math.min(heading[1].length, 6)
    return `<div class="md-h md-h${level}">${richText(heading[2])}</div>`
  }
  if (CODE_FENCE_RE.test(p)) {
    const lines = p.split('\n')
    const inner = lines.slice(1).join('\n').replace(/```\s*$/, '')
    return `<pre class="md-code"><code>${escapeHtml(inner)}</code></pre>`
  }
  const table = tableHtml(p)
  if (table) return table
  const list = listHtml(p)
  if (list) return list
  if (opts.optionLetters) {
    const opt = p.match(/^\(([A-Z])\)\s*([\s\S]*)$/)
    if (opt) {
      return `<div class="md-opt"><span class="opt-letter">（${opt[1]}）</span><span>${richText(opt[2])}</span></div>`
    }
  }
  if (p.startsWith('$$') && p.endsWith('$$')) {
    return `<div class="q-math">${katexHtml(p.slice(2, -2), true)}</div>`
  }
  if (p.startsWith('>')) {
    const body = p
      .split('\n')
      .map((l) => l.replace(/^>\s?/, ''))
      .join('\n')
    return `<blockquote class="q-quote">${richText(body)}</blockquote>`
  }
  return `<p>${richText(p)}</p>`
}
