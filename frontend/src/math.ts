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

function richTextBase(src: string, single: boolean): string {
  const maths: { tex: string; display: boolean }[] = []
  let s = src.replace(/\$\$([\s\S]+?)\$\$/g, (_m, tex: string) => {
    maths.push({ tex, display: !single })
    return `${NUL}${maths.length - 1}${NUL}`
  })
  s = s.replace(/\$([^$\n]+?)\$/g, (_m, tex: string) => {
    maths.push({ tex, display: false })
    return `${NUL}${maths.length - 1}${NUL}`
  })
  s = escapeHtml(s)
  s = s.replace(/\*\*([\s\S]+?)\*\*/g, '<strong>$1</strong>')
  // Math stays a placeholder here and is rendered only in the restore step
  // below: rendering it earlier would expose KaTeX's HTML to this newline
  // replacement and break the newlines inside its internal SVG paths
  s = s.replace(/\n/g, single ? ' ' : '<br/>')
  return s.replace(new RegExp(`${NUL}(\\d+)${NUL}`, 'g'), (_m, i: string) => {
    const item = maths[Number(i)]
    if (!item) return ''
    return item.display
      ? `<div class="q-math">${katexHtml(item.tex, true)}</div>`
      : katexHtml(item.tex, false)
  })
}

export function paraHtml(p: string): string {
  const img = p.match(/^!\[\[(.+?\.svg)\]\]$/)
  if (img) {
    const name = img[1]
    return `<div class="q-svg-wrap"><img class="q-svg" src="/api/assets/${encodeURIComponent(name)}" alt="${escapeHtml(name)}" /></div>`
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
